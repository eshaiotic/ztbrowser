#include "engine_hooks.h"
#include "ztbrowser-engine-qt/src/engine_bridge.cxx.h"

#include <QBuffer>
#include <QFile>
#include <QGuiApplication>
#include <QIcon>
#include <QImage>
#include <QEventLoop>
#include <QNetworkAccessManager>
#include <QNetworkProxyFactory>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QQuickWebEngineProfile>
#include <QWebEngineCookieStore>
#include <QWebEngineUrlRequestInfo>
#include <QWebEngineUrlRequestInterceptor>
#include <QtWebEngineCore/qtwebenginecoreglobal.h>
#include <QtWebEngineQuick>
#include <atomic>

namespace {

/// Called by Chromium on its IO thread for every request; the decision is made in Rust.
class DecidingInterceptor : public QWebEngineUrlRequestInterceptor {
public:
    void interceptRequest(QWebEngineUrlRequestInfo &info) override {
        const QByteArray url = info.requestUrl().toEncoded();
        const QByteArray page = info.firstPartyUrl().toEncoded();
        const HookDecision decision = decide_request(
            rust::Str(url.constData(), static_cast<std::size_t>(url.size())),
            rust::Str(page.constData(), static_cast<std::size_t>(page.size())),
            static_cast<std::int32_t>(info.resourceType()), ++request_counter_);
        if (decision.action == 1) {
            info.block(true);
        } else if (decision.action == 2) {
            info.redirect(QUrl(QString::fromUtf8(decision.redirect_to.data(), static_cast<int>(decision.redirect_to.size()))));
        }
    }

private:
    std::atomic<std::uint64_t> request_counter_{0};
};

} // namespace

void initialize_web_engine(rust::Str chromium_flags) {
    if (!chromium_flags.empty()) {
        QByteArray flags = qgetenv("QTWEBENGINE_CHROMIUM_FLAGS");
        if (!flags.isEmpty()) {
            flags.append(' ');
        }
        flags.append(chromium_flags.data(), static_cast<qsizetype>(chromium_flags.size()));
        qputenv("QTWEBENGINE_CHROMIUM_FLAGS", flags);
    }
    QtWebEngineQuick::initialize();
}

void install_hooks(rust::Str storage_path, rust::Str cache_path) {
    QQuickWebEngineProfile *profile = QQuickWebEngineProfile::defaultProfile();
    // The directories are set before the profile becomes persistent: making it persistent first would create
    // the engine's files (history, favicons) in Qt's default location, outside the directory burning deletes.
    profile->setPersistentStoragePath(QString::fromUtf8(storage_path.data(), static_cast<int>(storage_path.size())));
    profile->setCachePath(QString::fromUtf8(cache_path.data(), static_cast<int>(cache_path.size())));
    // The default profile of QtWebEngine Quick keeps nothing on disk until it is made persistent.
    profile->setStorageName(QStringLiteral("ZTBrowser"));
    profile->setOffTheRecord(false);
    profile->setPersistentCookiesPolicy(QQuickWebEngineProfile::AllowPersistentCookies);
    profile->setUrlRequestInterceptor(new DecidingInterceptor);
    profile->cookieStore()->setCookieFilter([](const QWebEngineCookieStore::FilterRequest &request) {
        const QByteArray domain = request.origin.host().toUtf8();
        const QByteArray page = request.firstPartyUrl.host().toUtf8();
        return allow_cookie(rust::Str(domain.constData(), static_cast<std::size_t>(domain.size())),
                            rust::Str(page.constData(), static_cast<std::size_t>(page.size())), request.thirdParty);
    });
}

rust::String chromium_version() {
    return rust::String(qWebEngineChromiumVersion());
}

void set_window_icon(rust::Str resource_path) {
    QGuiApplication::setWindowIcon(
        QIcon(QString::fromUtf8(resource_path.data(), static_cast<int>(resource_path.size()))));
}

rust::String read_qt_resource(rust::Str path) {
    QFile file(QString::fromUtf8(path.data(), static_cast<int>(path.size())));
    if (!file.open(QIODevice::ReadOnly)) {
        return rust::String();
    }
    return rust::String(QString::fromUtf8(file.readAll()).toStdString());
}

rust::Vec<std::uint8_t> reencode_png(rust::Slice<const std::uint8_t> bytes) {
    rust::Vec<std::uint8_t> encoded;
    QImage image;
    if (!image.loadFromData(bytes.data(), static_cast<int>(bytes.size()), "PNG")) {
        return encoded;
    }
    QByteArray png;
    QBuffer buffer(&png);
    if (!buffer.open(QIODevice::WriteOnly) || !image.save(&buffer, "PNG")) {
        return encoded;
    }
    for (const char byte : png) {
        encoded.push_back(static_cast<std::uint8_t>(byte));
    }
    return encoded;
}

FetchedBytes fetch_bytes(rust::Str url, rust::Str etag, std::uint64_t max_bytes, std::int32_t timeout_ms) {
    FetchedBytes answer{0, rust::String(), rust::Vec<std::uint8_t>(), rust::String()};
    QNetworkProxyFactory::setUseSystemConfiguration(true);
    // A manager of its own for each download: nothing, not even a cookie, is kept between two of them.
    QNetworkAccessManager manager;
    QNetworkRequest request(QUrl(QString::fromUtf8(url.data(), static_cast<int>(url.size()))));
    request.setAttribute(QNetworkRequest::RedirectPolicyAttribute, QNetworkRequest::NoLessSafeRedirectPolicy);
    request.setAttribute(QNetworkRequest::CookieLoadControlAttribute, QNetworkRequest::Manual);
    request.setAttribute(QNetworkRequest::CookieSaveControlAttribute, QNetworkRequest::Manual);
    request.setTransferTimeout(timeout_ms);
    request.setRawHeader("User-Agent", "ZTBrowser");
    if (!etag.empty()) {
        request.setRawHeader("If-None-Match", QByteArray(etag.data(), static_cast<qsizetype>(etag.size())));
    }
    QNetworkReply *reply = manager.get(request);
    bool too_large = false;
    QObject::connect(reply, &QNetworkReply::downloadProgress, [&](qint64 received, qint64) {
        if (static_cast<std::uint64_t>(received) > max_bytes) {
            too_large = true;
            reply->abort();
        }
    });
    QEventLoop loop;
    QObject::connect(reply, &QNetworkReply::finished, &loop, &QEventLoop::quit);
    loop.exec();
    answer.status = reply->attribute(QNetworkRequest::HttpStatusCodeAttribute).toInt();
    if (too_large) {
        answer.status = 0;
        answer.error = rust::String("the body is larger than allowed");
    } else if (reply->error() != QNetworkReply::NoError && answer.status == 0) {
        answer.error = rust::String(reply->errorString().toStdString());
    } else {
        answer.etag = rust::String(reply->rawHeader("ETag").toStdString());
        const QByteArray body = reply->readAll();
        for (const char byte : body) {
            answer.body.push_back(static_cast<std::uint8_t>(byte));
        }
    }
    reply->deleteLater();
    return answer;
}
