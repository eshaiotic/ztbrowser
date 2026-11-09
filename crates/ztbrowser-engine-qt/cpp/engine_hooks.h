#pragma once
#include "rust/cxx.h"
#include <cstdint>

/// Defined by the generated bridge header, which includes this one.
struct FetchedBytes;

/// Must run before the QGuiApplication exists (a QtWebEngine requirement). `chromium_flags` are appended to the
/// flags Chromium starts with (after any the person set in `QTWEBENGINE_CHROMIUM_FLAGS`, so these win).
void initialize_web_engine(rust::Str chromium_flags);
/// Points the default web profile at its own directories and hands it the request interceptor and cookie
/// filter. Everything the engine stores goes under `storage_path` and `cache_path`, nowhere else.
void install_hooks(rust::Str storage_path, rust::Str cache_path);
/// Version of the Chromium inside QtWebEngine, such as `140.0.7339.207`.
rust::String chromium_version();
/// Gives every window the icon at `resource_path` (a Qt resource such as `:/icons/app.svg`); on Wayland and X11
/// alike this is what the window's title bar and the task bar show when no installed desktop entry names one.
void set_window_icon(rust::Str resource_path);
/// The text of a Qt resource (`:/qtwebchannel/qwebchannel.js`); empty when there is no such resource.
rust::String read_qt_resource(rust::Str path);
/// Decodes a PNG and encodes it again, so what reaches the desktop's icon loader is what Qt's decoder understood
/// and wrote itself, not the bytes a site sent. Empty when `bytes` are not a PNG Qt can decode.
rust::Vec<std::uint8_t> reencode_png(rust::Slice<const std::uint8_t> bytes);
/// Downloads `url` with a plain `GET`: no cookies, no identifier, redirects only to equally safe addresses, the
/// proxy the system names. Blocks the calling thread (which must not be the one running the interface) until
/// the answer is complete, `timeout_ms` has passed without progress, or more than `max_bytes` arrived.
FetchedBytes fetch_bytes(rust::Str url, rust::Str etag, std::uint64_t max_bytes, std::int32_t timeout_ms);
