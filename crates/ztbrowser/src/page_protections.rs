//! Everything WebKit injects into the pages of one profile: compiled rule lists and the content scope script.
//!
//! Several rule lists, each compiled separately because `ignore-previous-rules` only reaches back within its
//! own list: tracker blocking (the Tracker Data Set), HTTPS upgrades, tracking-parameter detection and the
//! ZTBrowser Guard lists. Compiling a large list takes seconds, so every compiled list is kept in a per-profile
//! `UserContentFilterStore` together with a fingerprint of what it was made from; an unchanged list is loaded
//! instead. A Guard list's source is not even built then: building it is the costly part.
//!
//! A Guard list's generic element hiding is not compiled into WebKit (see `GenericHiding`): it is stored next to
//! the compiled list, and Guard's page script asks for the part a page needs through a message handler of its
//! own script world.

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use ztbrowser_core::guard::generic_hiding::GenericHiding;
use ztbrowser_core::guard::generic_hiding_request::GenericHidingRequest;
use ztbrowser_core::guard::guard_rule_list::ConvertedGuardList;
use ztbrowser_core::guard::guard_rule_list::GuardRuleList;

use serde_json::Value;
use serde_json::json;
use webkit6::UserContentFilter;
use webkit6::UserContentFilterStore;
use webkit6::UserContentInjectedFrames;
use webkit6::UserContentManager;
use webkit6::UserScript;
use webkit6::UserScriptInjectionTime;
use ztbrowser_core::protection::content_blocker_rule::ContentBlockerRule;
use ztbrowser_core::protection::content_scope_message::ContentScopeMessage;
use ztbrowser_core::protection::content_scope_preferences::ContentScopePreferences;
use ztbrowser_core::protection::content_scope_script::ContentScopeScript;
use ztbrowser_core::protection::privacy_configuration::PrivacyConfiguration;
use ztbrowser_core::scrolling::smooth_wheel_scroll::SmoothWheelScroll;

use crate::smooth_wheel_script::SmoothWheelScript;
use crate::stored_file::StoredFile;

/// One named rule list, e.g. `trackers`.
pub struct RuleList {
    pub name: &'static str,
    pub rules: Vec<ContentBlockerRule>,
}

impl RuleList {
    /// The list as WebKit reads it. For the tracker list this takes a tenth of a second or more, which is why
    /// `ProfileSession` prepares that list on a worker thread.
    pub fn prepared(&self) -> PreparedRuleList {
        let source = Value::Array(self.rules.iter().map(ContentBlockerRule::to_json).collect()).to_string();
        PreparedRuleList {
            name: self.name,
            fingerprint: Self::fingerprint(&source),
            source: RuleSource::Built(source),
            is_empty: self.rules.is_empty(),
        }
    }

    fn fingerprint(source: &str) -> String {
        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}

/// A rule list ready for WebKit, with a fingerprint telling whether the stored compiled copy is current.
pub struct PreparedRuleList {
    name: &'static str,
    source: RuleSource,
    fingerprint: String,
    is_empty: bool,
}

impl PreparedRuleList {
    /// A Guard list, costly to build: `build` runs on a worker thread, and only when the stored compiled copy
    /// was not made from what `fingerprint` identifies.
    pub fn built_on_demand(
        name: &'static str,
        fingerprint: String,
        is_empty: bool,
        build: impl Fn() -> ConvertedGuardList + Send + Sync + 'static,
    ) -> Self {
        Self { name, source: RuleSource::OnDemand(Arc::new(build)), fingerprint, is_empty }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

/// The JSON WebKit compiles a list from.
enum RuleSource {
    Built(String),
    OnDemand(Arc<dyn Fn() -> ConvertedGuardList + Send + Sync>),
}

pub struct PageProtections {
    content_manager: UserContentManager,
    filter_store: UserContentFilterStore,
    filter_store_path: PathBuf,
    preferences: ContentScopePreferences,
    guard_script: String,
    smooth_wheel: SmoothWheelScript,
    is_wheel_smoothed: Cell<bool>,
    /// The generic hiding of the active Guard lists, by list name, which the page script's requests read.
    generic_hiding: Rc<RefCell<HashMap<String, GenericHiding>>>,
    /// Bumped by every activation, so that a slower read of an older one never lands.
    activations: Rc<Cell<u64>>,
}

impl PageProtections {
    /// DuckDuckGo Content Scope Scripts 20.0.0 (Apache-2.0), vendored and pinned by SHA-256 in the repository.
    const SCRIPT_TEMPLATE: &'static str = include_str!("../../../assets/content-scope-scripts/contentScope.js");
    /// The macOS app version that bundles this script; the privacy configuration gates features on it.
    pub const COMPATIBILITY_VERSION: &'static str = PrivacyConfiguration::COMPATIBILITY_VERSION;
    const LOG_DOMAIN: &'static str = "ztbrowser";
    /// ZTBrowser Guard's page script (generic element hiding), its script world and message handler.
    const GUARD_SCRIPT_TEMPLATE: &'static str = include_str!("../../../assets/guard/generic-hiding.js");
    const GUARD_WORLD: &'static str = "ztbrowser-guard";
    const GUARD_HANDLER: &'static str = "ztbrowserGuard";

    pub fn new(filter_store_path: &Path) -> Self {
        let content_manager = UserContentManager::new();
        let handler = ContentScopePreferences::MESSAGING_CONTEXT_NAME;
        content_manager.register_script_message_handler_with_reply(handler, None);
        content_manager.connect_script_message_with_reply_received(Some(handler), |_, message, reply| {
            let parsed = message.to_json(0).and_then(|text| serde_json::from_str(&text).ok()).unwrap_or(Value::Null);
            if let Some(context) = message.context() {
                let answer = ContentScopeMessage::new(&parsed).reply();
                reply.return_value(&javascriptcore::Value::new_string(&context, Some(&answer)));
            }
            true
        });
        let generic_hiding = Rc::new(RefCell::new(HashMap::new()));
        Self::answer_guard_requests(&content_manager, &generic_hiding);
        Self {
            content_manager,
            filter_store: UserContentFilterStore::new(&filter_store_path.to_string_lossy()),
            filter_store_path: filter_store_path.to_owned(),
            guard_script: Self::guard_script(),
            smooth_wheel: SmoothWheelScript::new(&SmoothWheelScroll::chromium_like()),
            is_wheel_smoothed: Cell::new(true),
            generic_hiding,
            activations: Rc::new(Cell::new(0)),
            preferences: ContentScopePreferences {
                global_privacy_control: true,
                session_key: glib::uuid_string_random().to_string(),
                message_secret: glib::uuid_string_random().to_string(),
                language: Self::user_language(),
                compatibility_version: Self::COMPATIBILITY_VERSION.to_owned(),
            },
        }
    }

    pub fn content_manager(&self) -> &UserContentManager {
        &self.content_manager
    }

    /// Replaces the content scope script; it runs at document start in every frame, in the page's world.
    /// Guard's page script runs beside it, in its own world.
    pub fn install_script(&self, configuration: &PrivacyConfiguration, user_unprotected_hosts: &[String]) {
        let source = ContentScopeScript::new(Self::SCRIPT_TEMPLATE).render(
            configuration,
            user_unprotected_hosts,
            &self.preferences,
        );
        let script =
            UserScript::new(&source, UserContentInjectedFrames::AllFrames, UserScriptInjectionTime::Start, &[], &[]);
        let guard = UserScript::for_world(
            &self.guard_script,
            UserContentInjectedFrames::AllFrames,
            UserScriptInjectionTime::Start,
            Self::GUARD_WORLD,
            &[],
            &[],
        );
        self.content_manager.remove_all_scripts();
        self.content_manager.add_script(&script);
        self.content_manager.add_script(&guard);
        if self.is_wheel_smoothed.get() {
            self.content_manager.add_script(&self.smooth_wheel.user_script());
        }
    }

    /// Whether pages loaded from now on get the smooth wheel script; [`Self::install_script`] applies it.
    pub fn set_wheel_smoothed(&self, is_smoothed: bool) {
        self.is_wheel_smoothed.set(is_smoothed);
    }

    /// The list compiled last time, if WebKit still has it. The store belongs to this profile alone, so this
    /// is the profile's own last list: it can protect pages while the current one is being built.
    pub async fn last_compiled(&self, name: &str) -> Option<UserContentFilter> {
        if !self.marker(name).exists() {
            return None;
        }
        self.filter_store.load_future(name).await.ok()
    }

    /// Compiles (or loads) every non-empty list; a list WebKit rejects is logged and left out.
    pub async fn compile(&self, lists: &[PreparedRuleList]) -> Vec<UserContentFilter> {
        let mut filters = Vec::new();
        for list in lists.iter().filter(|list| !list.is_empty) {
            match self.compiled(list).await {
                Ok(filter) => filters.push(filter),
                Err(error) => glib::g_critical!(Self::LOG_DOMAIN, "rule list {} rejected: {error}", list.name),
            }
        }
        filters
    }

    /// Splits `lists` into the compiled copies that are still current (loaded at once) and the lists that must
    /// be compiled again; empty lists are dropped.
    pub async fn current_or_stale(
        &self,
        lists: Vec<PreparedRuleList>,
    ) -> (Vec<UserContentFilter>, Vec<PreparedRuleList>) {
        let (mut current, mut stale) = (Vec::new(), Vec::new());
        for list in lists.into_iter().filter(|list| !list.is_empty) {
            match self.current(&list).await {
                Some(filter) => current.push(filter),
                None => stale.push(list),
            }
        }
        (current, stale)
    }

    /// Swaps in a new set of compiled lists; pages pick them up on their next load. The generic hiding of the
    /// Guard lists among them is read on a worker thread (a few megabytes of JSON) and answers requests from
    /// then on.
    pub fn activate(&self, filters: &[UserContentFilter]) {
        self.content_manager.remove_all_filters();
        for filter in filters {
            self.content_manager.add_filter(filter);
        }
        let activation = self.activations.get() + 1;
        self.activations.set(activation);
        let stored: Vec<(String, StoredFile)> = filters
            .iter()
            .filter_map(UserContentFilter::identifier)
            .map(|name| (name.to_string(), self.generic_hiding_file(&name)))
            .collect();
        let (generic_hiding, activations) = (Rc::clone(&self.generic_hiding), Rc::clone(&self.activations));
        glib::spawn_future_local(async move {
            let read = gio::spawn_blocking(move || {
                stored
                    .into_iter()
                    .filter_map(|(name, file)| {
                        file.read_text().and_then(|text| GenericHiding::from_json(&text)).map(|hiding| (name, hiding))
                    })
                    .collect::<HashMap<String, GenericHiding>>()
            });
            if let Ok(lists) = read.await
                && activations.get() == activation
            {
                *generic_hiding.borrow_mut() = lists;
            }
        });
    }

    /// The stored compiled copy of `list`, when it was made from what the list's fingerprint identifies.
    async fn current(&self, list: &PreparedRuleList) -> Option<UserContentFilter> {
        if self.marker(list.name).read_text().as_deref() != Some(list.fingerprint.as_str()) {
            return None;
        }
        self.filter_store.load_future(list.name).await.ok()
    }

    async fn compiled(&self, list: &PreparedRuleList) -> Result<UserContentFilter, glib::Error> {
        if let Some(filter) = self.current(list).await {
            return Ok(filter);
        }
        let (source, generic_hiding) = match &list.source {
            RuleSource::Built(source) => (glib::Bytes::from(source.as_bytes()), None),
            RuleSource::OnDemand(build) => {
                let build = Arc::clone(build);
                let (source, generic_hiding) = gio::spawn_blocking(move || {
                    let (source, generic_hiding) = build().into_parts();
                    (source, generic_hiding.to_json())
                })
                .await
                .map_err(|_| {
                    glib::Error::new(gio::IOErrorEnum::Failed, &format!("building rule list {} failed", list.name))
                })?;
                (glib::Bytes::from_owned(source), Some(generic_hiding))
            }
        };
        let filter = self.filter_store.save_future(list.name, &source).await?;
        if let Some(generic_hiding) = generic_hiding
            && let Err(error) = self.generic_hiding_file(list.name).write(generic_hiding.as_bytes())
        {
            glib::g_warning!(Self::LOG_DOMAIN, "cannot store the generic hiding of {}: {error}", list.name);
        }
        if let Err(error) = self.marker(list.name).write(list.fingerprint.as_bytes()) {
            glib::g_warning!(Self::LOG_DOMAIN, "cannot record rule list fingerprint: {error}");
        }
        Ok(filter)
    }

    /// Fingerprint of the source the stored compiled list `name` was made from.
    fn marker(&self, name: &str) -> StoredFile {
        StoredFile::new(self.filter_store_path.join(format!("{name}.fingerprint")))
    }

    /// The generic element hiding of the Guard list `name`, stored when the list was compiled.
    fn generic_hiding_file(&self, name: &str) -> StoredFile {
        StoredFile::new(self.filter_store_path.join(format!("{name}.generic-hiding.json")))
    }

    /// Answers Guard's page script, whose messages come from the page's web process: malformed ones get an
    /// empty answer.
    fn answer_guard_requests(
        content_manager: &UserContentManager,
        lists: &Rc<RefCell<HashMap<String, GenericHiding>>>,
    ) {
        content_manager.register_script_message_handler_with_reply(Self::GUARD_HANDLER, Some(Self::GUARD_WORLD));
        let lists = Rc::clone(lists);
        content_manager.connect_script_message_with_reply_received(
            Some(Self::GUARD_HANDLER),
            move |_, message, reply| {
                let parsed =
                    message.to_json(0).and_then(|text| serde_json::from_str(&text).ok()).unwrap_or(Value::Null);
                let answer = GenericHidingRequest::from_json(&parsed)
                    .map_or_else(|| json!({"selectors": [], "unkeyed": []}), |request| request.answer(&lists.borrow()));
                if let Some(context) = message.context() {
                    reply.return_value(&javascriptcore::Value::new_string(&context, Some(&answer.to_string())));
                }
                true
            },
        );
    }

    /// The page script with the Guard lists' names and probe element filled in.
    fn guard_script() -> String {
        let lists = Value::from(GuardRuleList::ALL.iter().map(|list| list.name()).collect::<Vec<_>>()).to_string();
        Self::GUARD_SCRIPT_TEMPLATE
            .replace("/*GUARD_LISTS*/[]", &lists)
            .replace("/*GUARD_PROBE_ELEMENT*/", GuardRuleList::PROBE_ELEMENT)
            .replace("/*GUARD_HANDLER*/", Self::GUARD_HANDLER)
    }

    /// Two-letter language of the user interface (`pl` from `pl_PL.UTF-8`).
    fn user_language() -> String {
        glib::language_names()
            .iter()
            .map(|name| name.as_str().chars().take_while(char::is_ascii_alphabetic).collect::<String>())
            .find(|language| language.len() == 2)
            .unwrap_or_else(|| "en".to_owned())
    }
}
