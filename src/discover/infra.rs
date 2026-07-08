//!
//! infra — THE INFRA GRAPH IS A LOCK: the deployment's meaning, declared, frozen, and
//! judged against what the cloud APIs can read back.
//!
//! Every incident class this module encodes lived OUTSIDE any repository, as dashboard
//! state or convention: a bucket's CORS allow-list that still named a migrated-away
//! origin (the law "a bucket must allow every origin that presigns against it" existed,
//! the seam existed, neither was declared — the refusal came from a customer's browser
//! weeks later); a build command living in a dashboard text box, the exact shape the
//! pipeline-is-a-lock brick killed for ci.yml; a surface running for a day on
//! placeholder secrets because no inventory said which secret NAMES it must hold; store
//! prefixes whose meanings (`ephemeral`, `locked`) were a regex in one file and prose
//! in another, re-encoded by hand wherever a probe needed them.
//!
//! This module extends the declaration discipline one level down, on the perimeter
//! pattern (`discover::perimeter` — declare, render, read the live state back, refuse
//! by name):
//!
//!   - [`Infra`] declares the graph: surfaces (origins + the build that produces them),
//!     stores whose roots carry a [`Meaning`] class, seams (who presigns what into
//!     where; who reads what), credential NAMES per surface (names, never values),
//!     authorities (who may mint, over which roots), and cadences.
//!   - [`Infra::coherent`] judges the declaration against itself — a seam naming an
//!     undeclared store, a presign landing outside any declared root: refused by name
//!     before any API is consulted.
//!   - [`Infra::judge`] holds a [`LiveInfra`] — field reads a consumer extracts from
//!     its cloud APIs — to the declared laws: *cors-covers-origins*, *secrets-census*,
//!     *build-command-is-derived*. Floor semantics as in the perimeter: extra live
//!     origins or secrets are not drift; the build command is the one exact match.
//!   - [`Infra::floor`] is the register floor where no API reaches: every declared
//!     fact the judge cannot see (authorities, cadences, root meanings) must carry a
//!     hand-ratified line in an `infra.register` — declared-but-unjudgeable is
//!     disclosed as exactly that, never silently assumed held.
//!   - [`Infra::lock`] freezes the render as `spec/<system>.infra.spec`; a surface
//!     migration becomes a declaration edit whose laws re-judge on the spot.
//!
//! What derives instead of being re-encoded: [`Infra::ephemeral_prefixes`] gives a
//! probe the writable-without-consequence roots (and their TTLs) straight from the
//! declaration — the hand-maintained refusal lists retire.
//!
//! Honest frame: the judge holds DECLARED facts against READABLE state — a seam nobody
//! declares is invisible (the same census honesty as everywhere). Live reads need
//! credentials and network at judgment time, so the judging gate's own credential is a
//! line in the very census it checks — that circularity is why the register floor
//! exists from day one. The extraction (which cloud API, which fields) belongs to the
//! consumer, like `examples/perimeter.rs` belongs to this repo; only the judgment
//! lives here, where the probes reach.

use std::path::PathBuf;

use spec_lock::{Lock, Register};

/// The MEANING of a store root — the semantics every probe and reaper otherwise
/// re-encodes by hand. Ephemeral roots carry their TTL by construction and locked
/// roots cannot carry one: half of prefix-discipline is unrepresentable as drift.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Meaning {
    /// Reaper-deleted this many hours after write; safe for probes to write under.
    Ephemeral {
        /// Hours from write to reap — the cadence the reaper must honour.
        ttl_hours: u64,
    },
    /// Customer data under an indefinite lock; nothing automated deletes here.
    Locked,
    /// Work in progress — writable, promotable, not yet under lock.
    Drafts,
}

impl Meaning {
    /// One word plus the TTL where one exists — the render and register vocabulary.
    pub fn describe(&self) -> String {
        match self {
            Meaning::Ephemeral { ttl_hours } => format!("ephemeral({ttl_hours}h)"),
            Meaning::Locked => "locked".to_string(),
            Meaning::Drafts => "drafts".to_string(),
        }
    }
}

/// A deployed surface: its origins and the build that produces it (`None` for a
/// surface nothing builds). The build COMMAND is declared here so the dashboard copy
/// can be judged against it — the ci.yml pattern applied one level down.
pub struct Surface {
    /// The surface's name in the declaration.
    pub name: String,
    /// Origins this surface serves from — what CORS allow-lists must cover.
    pub origins: Vec<String>,
    /// The build command the platform must run, if the surface is built.
    pub build: Option<String>,
}

/// A root inside a store: a prefix and what writes under it MEAN.
pub struct Root {
    /// The prefix, as writers address it (e.g. `test/`).
    pub prefix: String,
    /// The retention/consequence class of everything under the prefix.
    pub meaning: Meaning,
}

/// A store (bucket) and the declared meanings of its roots.
pub struct Store {
    /// The store's name in the declaration.
    pub name: String,
    /// Every root a declared writer may address — an undeclared prefix refuses.
    pub roots: Vec<Root>,
}

/// A seam between a surface and a store — the edges the laws quantify over.
pub enum Seam {
    /// The surface presigns PUT into the store under the prefix. This is the edge the
    /// cors-covers-origins law reads: the store must allow the surface's origins.
    Presign {
        /// The presigning surface, by declared name.
        surface: String,
        /// The receiving store, by declared name.
        store: String,
        /// The root prefix written under — must be a declared [`Root`].
        prefix: String,
    },
    /// The surface reads the store (listings, downloads).
    Reads {
        /// The reading surface, by declared name.
        surface: String,
        /// The store read, by declared name.
        store: String,
    },
}

/// The secret NAMES a surface must hold — names only, never values. "Configured" and
/// "placeholder" stay distinguishable one level up: the census says which names must
/// exist; whether a value works is the post-deploy probe's job, not this inventory's.
pub struct Credential {
    /// The surface holding the secrets, by declared name.
    pub surface: String,
    /// The secret names the platform's env must contain.
    pub names: Vec<String>,
}

/// An authority: who may mint what, over which roots. No API reads this — it lives on
/// the register floor — but declaring it makes replacing the mechanism a declaration
/// edit instead of archaeology.
pub struct Authority {
    /// The authority's name in the declaration.
    pub name: String,
    /// What it authorizes minting (tokens, sessions, uploads).
    pub mints: String,
    /// The root prefixes the minted capability may reach.
    pub over: Vec<String>,
}

/// A cadence: something the deployment does on a clock (a reaper, a post-deploy
/// probe). Register-floor facts, like authorities.
pub struct Cadence {
    /// The cadence's name in the declaration.
    pub name: String,
    /// When it runs and what it does — prose, judged by ratification.
    pub schedule: String,
}

/// The declared infra graph of one system — what `spec/<system>.infra.spec` freezes.
pub struct Infra {
    /// The system's name; the lock file is `spec/<slug>.infra.spec`.
    pub system: String,
    /// Deployed surfaces.
    pub surfaces: Vec<Surface>,
    /// Stores and their root meanings.
    pub stores: Vec<Store>,
    /// The declared edges.
    pub seams: Vec<Seam>,
    /// Secret-name censuses, per surface.
    pub credentials: Vec<Credential>,
    /// Who may mint, over what.
    pub authorities: Vec<Authority>,
    /// What runs on a clock.
    pub cadences: Vec<Cadence>,
}

/// What the world reports — field reads a consumer's extractor pulls from its cloud
/// APIs (CORS rules, secret-name listings, project build settings). An ABSENT entry
/// means the endpoint could not be read; the judge refuses that by name, never
/// assumes it held.
#[derive(Default)]
pub struct LiveInfra {
    /// Store name → the origins its CORS allow-list currently permits.
    pub cors: Vec<(String, Vec<String>)>,
    /// Surface name → the secret names its platform env currently holds (names only).
    pub secret_names: Vec<(String, Vec<String>)>,
    /// Surface name → the build command the dashboard currently runs.
    pub build_commands: Vec<(String, String)>,
}

impl LiveInfra {
    fn lookup<'a, T>(map: &'a [(String, T)], key: &str) -> Option<&'a T> {
        map.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }
}

impl Infra {
    fn surface(&self, name: &str) -> Option<&Surface> {
        self.surfaces.iter().find(|s| s.name == name)
    }

    fn store(&self, name: &str) -> Option<&Store> {
        self.stores.iter().find(|s| s.name == name)
    }

    /// The roots a probe may write under without consequence, with their TTLs — the
    /// derivation that retires hand-encoded refusal lists in e2e probes.
    pub fn ephemeral_prefixes(&self, store: &str) -> Vec<(String, u64)> {
        self.store(store)
            .map(|s| {
                s.roots
                    .iter()
                    .filter_map(|r| match r.meaning {
                        Meaning::Ephemeral { ttl_hours } => Some((r.prefix.clone(), ttl_hours)),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Judge the declaration AGAINST ITSELF — no API consulted. Every edge must land
    /// on declared nodes and every presign under a declared root (prefix-discipline's
    /// declarable half; the TTL half is unrepresentable by construction — see
    /// [`Meaning`]). `Ok` carries the held facts; `Err` refuses each incoherence by
    /// name.
    pub fn coherent(&self) -> Result<Vec<String>, Vec<String>> {
        let mut held = Vec::new();
        let mut refusals = Vec::new();
        for seam in &self.seams {
            let (surface, store) = match seam {
                Seam::Presign { surface, store, .. } => (surface, store),
                Seam::Reads { surface, store } => (surface, store),
            };
            if self.surface(surface).is_none() {
                refusals.push(format!(
                    "a seam names surface `{surface}`, which is not declared — an edge \
                     cannot land on a node the graph does not have"
                ));
            }
            if self.store(store).is_none() {
                refusals.push(format!(
                    "a seam names store `{store}`, which is not declared — an edge \
                     cannot land on a node the graph does not have"
                ));
                continue;
            }
            if let Seam::Presign {
                surface,
                store,
                prefix,
            } = seam
            {
                let declared_root = self
                    .store(store)
                    .is_some_and(|s| s.roots.iter().any(|r| r.prefix == *prefix));
                if declared_root {
                    held.push(format!(
                        "seam: `{surface}` presigns into `{store}` under `{prefix}`"
                    ));
                } else {
                    refusals.push(format!(
                        "`{surface}` presigns into `{store}` under `{prefix}`, but \
                         `{prefix}` is not a declared root of `{store}` — a writer \
                         writes only under declared prefixes"
                    ));
                }
            } else {
                held.push(format!("seam: `{surface}` reads `{store}`"));
            }
        }
        for credential in &self.credentials {
            if self.surface(&credential.surface).is_none() {
                refusals.push(format!(
                    "a credential census names surface `{}`, which is not declared",
                    credential.surface
                ));
            }
        }
        for authority in &self.authorities {
            for prefix in &authority.over {
                let declared = self
                    .stores
                    .iter()
                    .any(|s| s.roots.iter().any(|r| r.prefix == *prefix));
                if declared {
                    held.push(format!(
                        "authority `{}` reaches declared root `{prefix}`",
                        authority.name
                    ));
                } else {
                    refusals.push(format!(
                        "authority `{}` claims reach over `{prefix}`, which is not a \
                         declared root of any store",
                        authority.name
                    ));
                }
            }
        }
        if refusals.is_empty() {
            Ok(held)
        } else {
            Err(refusals)
        }
    }

    /// Hold the LIVE state to the declared laws. Refusals name the store, the origin,
    /// the surface, the secret — the incident report written BEFORE the incident.
    /// An incoherent declaration refuses before any live fact is consulted.
    pub fn judge(&self, live: &LiveInfra) -> Result<Vec<String>, Vec<String>> {
        let mut held = self.coherent()?;
        let mut violations = Vec::new();

        // cors-covers-origins: every store presigned against must allow every origin
        // of every surface that presigns into it. Extra live origins are NOT drift
        // under floor semantics; the law is coverage.
        for store in &self.stores {
            let mut needed: Vec<&String> = Vec::new();
            for seam in &self.seams {
                if let Seam::Presign {
                    surface, store: s, ..
                } = seam
                {
                    if *s == store.name {
                        if let Some(surface) = self.surface(surface) {
                            needed.extend(surface.origins.iter());
                        }
                    }
                }
            }
            if needed.is_empty() {
                continue;
            }
            match LiveInfra::lookup(&live.cors, &store.name) {
                None => violations.push(format!(
                    "the CORS allow-list of `{}` could not be READ — refused by name, \
                     never assumed to cover its presigning origins",
                    store.name
                )),
                Some(allowed) => {
                    for origin in needed {
                        if allowed.iter().any(|a| a == origin) {
                            held.push(format!(
                                "cors: `{}` allows presigning origin `{origin}`",
                                store.name
                            ));
                        } else {
                            violations.push(format!(
                                "store `{}` does not allow origin `{origin}`, which \
                                 presigns against it — uploads from that origin will \
                                 fail in the customer's browser",
                                store.name
                            ));
                        }
                    }
                }
            }
        }

        // secrets-census: every declared secret NAME must exist on its surface.
        // Extra configured names are not drift; a missing name is a surface running
        // on absence.
        for credential in &self.credentials {
            match LiveInfra::lookup(&live.secret_names, &credential.surface) {
                None => violations.push(format!(
                    "the secret names of `{}` could not be READ — refused by name, \
                     never assumed configured",
                    credential.surface
                )),
                Some(configured) => {
                    for name in &credential.names {
                        if configured.iter().any(|c| c == name) {
                            held.push(format!("secret: `{}` holds `{name}`", credential.surface));
                        } else {
                            violations.push(format!(
                                "surface `{}` does not hold secret `{name}` — declared \
                                 in the census, absent from the platform",
                                credential.surface
                            ));
                        }
                    }
                }
            }
        }

        // build-command-is-derived: the dashboard's build command equals the declared
        // render, exactly — the one exact match, like the perimeter's approvals.
        for surface in &self.surfaces {
            let Some(declared) = &surface.build else {
                continue;
            };
            match LiveInfra::lookup(&live.build_commands, &surface.name) {
                None => violations.push(format!(
                    "the build command of `{}` could not be READ — refused by name, \
                     never assumed derived",
                    surface.name
                )),
                Some(command) if command == declared => {
                    held.push(format!(
                        "build: `{}` runs the declared command",
                        surface.name
                    ));
                }
                Some(command) => violations.push(format!(
                    "the build command of `{}` is `{command}`, declared `{declared}` — \
                     a gate defined where no drift gate can see it, unless this one",
                    surface.name
                )),
            }
        }

        if violations.is_empty() {
            Ok(held)
        } else {
            Err(violations)
        }
    }

    /// The declared facts NO API reads — authorities, cadences, root meanings. Each is
    /// a key the register floor must ratify: declared-but-unjudgeable is disclosed,
    /// never silently assumed held.
    pub fn unjudgeable(&self) -> Vec<String> {
        let mut keys = Vec::new();
        for store in &self.stores {
            for root in &store.roots {
                keys.push(format!(
                    "meaning {}/{} = {}",
                    store.name,
                    root.prefix,
                    root.meaning.describe()
                ));
            }
        }
        for authority in &self.authorities {
            keys.push(format!("authority {}", authority.name));
        }
        for cadence in &self.cadences {
            keys.push(format!("cadence {}", cadence.name));
        }
        keys
    }

    /// The register floor: every unjudgeable declared fact must carry a hand-ratified
    /// line (the [`Register`] grammar, set-difference drift). A new unjudgeable fact
    /// wants a justification; a ratified line for a fact no longer declared wants
    /// deleting — a stale exception is a lie the register tells forever.
    pub fn floor(&self, register: &Register) -> Result<(), String> {
        let keys = self.unjudgeable();
        register.check(keys.iter().map(|k| k.as_str()))
    }

    /// The human-readable graph — what `spec/<system>.infra.spec` freezes.
    pub fn render(&self) -> String {
        let mut out = format!(
            "# the infra graph of `{}`, DECLARED — surfaces, stores, seams, credentials\n\
             # (names only), authorities, cadences. Judged laws: cors-covers-origins,\n\
             # secrets-census, build-command-is-derived (against live API reads); the\n\
             # rest lives on the register floor (`{}.infra.register`) — declared but\n\
             # unjudgeable, disclosed as exactly that. Regenerate with\n\
             # `cargo run --example freeze_infra`.\n\n",
            self.system,
            self.slug()
        );
        for surface in &self.surfaces {
            out.push_str(&format!(
                "surface {} — origins: {}; build: {}\n",
                surface.name,
                surface.origins.join(", "),
                surface
                    .build
                    .as_deref()
                    .unwrap_or("(none — nothing builds it)")
            ));
        }
        for store in &self.stores {
            out.push_str(&format!("store {} — roots:\n", store.name));
            for root in &store.roots {
                out.push_str(&format!(
                    "  {} means {}\n",
                    root.prefix,
                    root.meaning.describe()
                ));
            }
        }
        for seam in &self.seams {
            match seam {
                Seam::Presign {
                    surface,
                    store,
                    prefix,
                } => out.push_str(&format!(
                    "seam: {surface} presigns PUT into {store} under {prefix}\n"
                )),
                Seam::Reads { surface, store } => {
                    out.push_str(&format!("seam: {surface} reads {store}\n"))
                }
            }
        }
        for credential in &self.credentials {
            out.push_str(&format!(
                "credentials of {}: {}\n",
                credential.surface,
                credential.names.join(", ")
            ));
        }
        for authority in &self.authorities {
            out.push_str(&format!(
                "authority {} — mints {}; reaches: {}\n",
                authority.name,
                authority.mints,
                authority.over.join(", ")
            ));
        }
        for cadence in &self.cadences {
            out.push_str(&format!(
                "cadence {} — {}\n",
                cadence.name, cadence.schedule
            ));
        }
        out
    }

    fn slug(&self) -> String {
        self.system
            .chars()
            .map(|c| if c == ' ' { '-' } else { c })
            .collect()
    }

    /// `spec/<slug>.infra.spec` — the graph, frozen under the same lock discipline as
    /// every other artifact.
    pub fn lock(&self) -> Lock {
        Lock {
            name: format!("{} infra", self.system),
            path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("spec")
                .join(format!("{}.infra.spec", self.slug())),
            live: self.render(),
        }
    }

    /// The register floor's committed file, next to the spec lock.
    pub fn register(&self) -> Register {
        Register {
            name: format!("{} infra floor", self.system),
            path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("spec")
                .join(format!("{}.infra.register", self.slug())),
        }
    }

    /// The EXEMPLAR deployment — the first consumer's shape with the names washed out
    /// (field reports describe outside projects by what they taught, never by name):
    /// two surfaces, one store with three root meanings, four seams, six secret names,
    /// one authority, two cadences. Committed here so the whole path — coherence,
    /// judgment, floor, freeze — runs against a real declaration on every test run.
    pub fn exemplar() -> Infra {
        let secret_names = || {
            vec![
                "OAUTH_CLIENT_SECRET".to_string(),
                "STORE_ACCESS_KEY_ID".to_string(),
                "STORE_SECRET_ACCESS_KEY".to_string(),
            ]
        };
        Infra {
            system: "exemplar".to_string(),
            surfaces: vec![
                Surface {
                    name: "app (prod)".to_string(),
                    origins: vec!["https://app.example".to_string()],
                    build: Some("npm ci && npm run build".to_string()),
                },
                Surface {
                    name: "app (preview)".to_string(),
                    origins: vec!["https://preview.app.example".to_string()],
                    build: Some("npm ci && npm run build".to_string()),
                },
            ],
            stores: vec![Store {
                name: "intake".to_string(),
                roots: vec![
                    Root {
                        prefix: "test/".to_string(),
                        meaning: Meaning::Ephemeral { ttl_hours: 24 },
                    },
                    Root {
                        prefix: "drafts/".to_string(),
                        meaning: Meaning::Drafts,
                    },
                    Root {
                        prefix: "v1/".to_string(),
                        meaning: Meaning::Locked,
                    },
                ],
            }],
            seams: vec![
                Seam::Presign {
                    surface: "app (prod)".to_string(),
                    store: "intake".to_string(),
                    prefix: "v1/".to_string(),
                },
                Seam::Presign {
                    surface: "app (preview)".to_string(),
                    store: "intake".to_string(),
                    prefix: "test/".to_string(),
                },
                Seam::Reads {
                    surface: "app (prod)".to_string(),
                    store: "intake".to_string(),
                },
                Seam::Reads {
                    surface: "app (preview)".to_string(),
                    store: "intake".to_string(),
                },
            ],
            credentials: vec![
                Credential {
                    surface: "app (prod)".to_string(),
                    names: secret_names(),
                },
                Credential {
                    surface: "app (preview)".to_string(),
                    names: secret_names(),
                },
            ],
            authorities: vec![Authority {
                name: "test-token mint".to_string(),
                mints: "test tokens (operator org membership — a declared holdover)".to_string(),
                over: vec!["test/".to_string()],
            }],
            cadences: vec![
                Cadence {
                    name: "reaper".to_string(),
                    schedule: "daily — deletes ephemeral roots past their TTL".to_string(),
                },
                Cadence {
                    name: "post-deploy probe".to_string(),
                    schedule: "every deploy — exercises the presign seams end to end".to_string(),
                },
            ],
        }
    }
}

#[cfg(test)]
mod probes {
    use super::*;

    /// A live state matching the exemplar's floor exactly.
    fn applied() -> LiveInfra {
        LiveInfra {
            cors: vec![(
                "intake".to_string(),
                vec![
                    "https://app.example".to_string(),
                    "https://preview.app.example".to_string(),
                ],
            )],
            secret_names: vec![
                (
                    "app (prod)".to_string(),
                    vec![
                        "OAUTH_CLIENT_SECRET".to_string(),
                        "STORE_ACCESS_KEY_ID".to_string(),
                        "STORE_SECRET_ACCESS_KEY".to_string(),
                    ],
                ),
                (
                    "app (preview)".to_string(),
                    vec![
                        "OAUTH_CLIENT_SECRET".to_string(),
                        "STORE_ACCESS_KEY_ID".to_string(),
                        "STORE_SECRET_ACCESS_KEY".to_string(),
                    ],
                ),
            ],
            build_commands: vec![
                (
                    "app (prod)".to_string(),
                    "npm ci && npm run build".to_string(),
                ),
                (
                    "app (preview)".to_string(),
                    "npm ci && npm run build".to_string(),
                ),
            ],
        }
    }

    /// The exemplar is coherent, and the derivations replace hand-encoded lists: the
    /// ephemeral prefixes (with TTLs) come from the declaration, not a probe's regex.
    #[test]
    fn the_exemplar_coheres_and_the_meanings_derive() {
        let infra = Infra::exemplar();
        let held = infra.coherent().expect("the exemplar declaration coheres");
        assert!(held
            .iter()
            .any(|h| h.contains("`app (prod)` presigns into `intake` under `v1/`")));
        assert_eq!(
            infra.ephemeral_prefixes("intake"),
            vec![("test/".to_string(), 24)]
        );
        assert_eq!(infra.ephemeral_prefixes("no such store"), vec![]);
    }

    /// Each incoherence refuses by name, ONE ARM AT A TIME: an undeclared surface, an
    /// undeclared store, a presign outside any declared root, a credential census on
    /// an undeclared surface, an authority over an undeclared prefix.
    #[test]
    fn each_incoherence_refuses_by_name() {
        let mut unknown_surface = Infra::exemplar();
        unknown_surface.seams.push(Seam::Reads {
            surface: "phantom".to_string(),
            store: "intake".to_string(),
        });
        let refusals = unknown_surface.coherent().unwrap_err();
        assert!(refusals.iter().any(|r| r.contains("surface `phantom`")));

        let mut unknown_store = Infra::exemplar();
        unknown_store.seams.push(Seam::Reads {
            surface: "app (prod)".to_string(),
            store: "phantom".to_string(),
        });
        let refusals = unknown_store.coherent().unwrap_err();
        assert!(refusals.iter().any(|r| r.contains("store `phantom`")));

        let mut stray_prefix = Infra::exemplar();
        stray_prefix.seams.push(Seam::Presign {
            surface: "app (prod)".to_string(),
            store: "intake".to_string(),
            prefix: "attic/".to_string(),
        });
        let refusals = stray_prefix.coherent().unwrap_err();
        assert!(refusals
            .iter()
            .any(|r| r.contains("`attic/` is not a declared root")));

        let mut stray_census = Infra::exemplar();
        stray_census.credentials.push(Credential {
            surface: "phantom".to_string(),
            names: vec!["X".to_string()],
        });
        let refusals = stray_census.coherent().unwrap_err();
        assert!(refusals
            .iter()
            .any(|r| r.contains("credential census names surface `phantom`")));

        let mut stray_authority = Infra::exemplar();
        stray_authority.authorities.push(Authority {
            name: "wide".to_string(),
            mints: "anything".to_string(),
            over: vec!["everything/".to_string()],
        });
        let refusals = stray_authority.coherent().unwrap_err();
        assert!(refusals
            .iter()
            .any(|r| r.contains("authority `wide` claims reach over `everything/`")));
    }

    /// The applied floor holds, and floor semantics carry down: extra live origins and
    /// extra configured secrets are NOT drift — stricter coverage is never a lie.
    #[test]
    fn an_applied_floor_holds_and_extra_coverage_is_not_drift() {
        let infra = Infra::exemplar();
        let held = infra.judge(&applied()).expect("the applied floor holds");
        assert!(held
            .iter()
            .any(|h| h.contains("cors: `intake` allows presigning origin")));
        assert!(held
            .iter()
            .any(|h| h == "secret: `app (prod)` holds `OAUTH_CLIENT_SECRET`"));
        assert!(held
            .iter()
            .any(|h| h == "build: `app (prod)` runs the declared command"));

        let mut wider = applied();
        wider.cors[0].1.push("https://ops.example".to_string());
        wider.secret_names[0].1.push("AN_EXTRA_SECRET".to_string());
        assert!(infra.judge(&wider).is_ok(), "extra coverage is not drift");
    }

    /// Every departure refuses by name — bucket AND origin, surface AND secret,
    /// declared AND live build command, and each unreadable endpoint separately.
    #[test]
    fn every_departure_refuses_by_name() {
        let infra = Infra::exemplar();

        // The migrated-origin incident: one origin dropped from the allow-list while
        // the other stays — the refusal names the missing origin, never the held one.
        let mut migrated = applied();
        migrated.cors[0].1.retain(|o| o != "https://app.example");
        let violations = infra.judge(&migrated).unwrap_err();
        assert!(violations
            .iter()
            .any(|v| v.contains("store `intake` does not allow origin `https://app.example`")));
        assert!(
            !violations
                .iter()
                .any(|v| v.contains("`https://preview.app.example`")),
            "the covered origin must not be reported: {violations:#?}"
        );

        // The placeholder-secrets incident: one name missing on one surface.
        let mut placeholder = applied();
        placeholder.secret_names[1]
            .1
            .retain(|n| n != "STORE_SECRET_ACCESS_KEY");
        let violations = infra.judge(&placeholder).unwrap_err();
        assert!(violations.iter().any(|v| v
            .contains("surface `app (preview)` does not hold secret `STORE_SECRET_ACCESS_KEY`")));
        assert!(
            !violations.iter().any(|v| v.contains("app (prod)")),
            "the fully configured surface must not be reported: {violations:#?}"
        );

        // The dashboard build command drifted from the declaration.
        let mut drifted = applied();
        drifted.build_commands[0].1 = "npm run build".to_string();
        let violations = infra.judge(&drifted).unwrap_err();
        assert!(violations.iter().any(|v| {
            v.contains("build command of `app (prod)` is `npm run build`")
                && v.contains("declared `npm ci && npm run build`")
        }));

        // Nothing readable at all: every endpoint refuses separately, by name.
        let violations = infra.judge(&LiveInfra::default()).unwrap_err();
        assert!(violations
            .iter()
            .any(|v| v.contains("CORS allow-list of `intake` could not be READ")));
        assert!(violations
            .iter()
            .any(|v| v.contains("secret names of `app (prod)` could not be READ")));
        assert!(violations
            .iter()
            .any(|v| v.contains("build command of `app (preview)` could not be READ")));
    }

    /// An incoherent declaration refuses BEFORE any live fact is consulted.
    #[test]
    fn an_incoherent_declaration_never_reaches_the_live_judge() {
        let mut broken = Infra::exemplar();
        broken.seams.push(Seam::Reads {
            surface: "phantom".to_string(),
            store: "intake".to_string(),
        });
        let violations = broken.judge(&applied()).unwrap_err();
        assert!(violations.iter().any(|r| r.contains("surface `phantom`")));
    }

    /// The register floor covers EXACTLY the unjudgeable facts: root meanings,
    /// authorities, cadences. A new unjudgeable fact drifts as a new finding; a line
    /// for a fact no longer declared drifts as resolved.
    #[test]
    fn the_register_floor_covers_exactly_the_unjudgeable_facts() {
        let infra = Infra::exemplar();
        assert_eq!(
            infra.unjudgeable(),
            vec![
                "meaning intake/test/ = ephemeral(24h)",
                "meaning intake/drafts/ = drafts",
                "meaning intake/v1/ = locked",
                "authority test-token mint",
                "cadence reaper",
                "cadence post-deploy probe",
            ]
        );
        infra
            .floor(&infra.register())
            .expect("the committed exemplar register covers its unjudgeable facts");

        let mut grown = Infra::exemplar();
        grown.cadences.push(Cadence {
            name: "nightly export".to_string(),
            schedule: "nightly".to_string(),
        });
        let drift = grown.floor(&grown.register()).unwrap_err();
        assert!(drift.contains("new finding(s)"));
        assert!(drift.contains("cadence nightly export"));

        let mut shrunk = Infra::exemplar();
        shrunk.authorities.clear();
        let drift = shrunk.floor(&shrunk.register()).unwrap_err();
        assert!(drift.contains("resolved"));
        assert!(drift.contains("authority test-token mint"));
    }

    /// The committed exemplar lock is FRESH — the declaration and its spec move
    /// together or the build refuses.
    #[test]
    fn the_committed_exemplar_lock_is_fresh() {
        let infra = Infra::exemplar();
        if let Err(stale) = spec_lock::check(&[infra.lock()]) {
            panic!(
                "the infra lock drifted: {}. Regenerate with \
                 `cargo run --example freeze_infra` and ratify the diff.",
                stale.join(", ")
            );
        }
    }
}
