use crate::helpers::ProtoResource;
use crate::printer::{format_value, Printer};
use chrono::{DateTime, NaiveDateTime};
use clap::Args;
use miette::IntoDiagnostic;
use proto_core::{Id, PluginLocator, ProtoToolConfig, ToolManifest, UnresolvedVersionSpec};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::Serialize;
use starbase::system;
use starbase_styles::color;
use starbase_utils::json;
use tokio::sync::Mutex;
use tracing::info;

#[derive(Serialize)]
pub struct PluginItem<'a> {
    name: String,
    locator: Option<PluginLocator>,
    config: Option<&'a ProtoToolConfig>,
    manifest: ToolManifest,
}

#[derive(Args, Clone, Debug)]
pub struct ListPluginsArgs {
    #[arg(help = "ID of plugins to list")]
    ids: Vec<Id>,

    #[arg(long, help = "Include resolved aliases in the output")]
    aliases: bool,

    #[arg(long, help = "Print the list in JSON format")]
    json: bool,

    #[arg(long, help = "Include installed versions in the output")]
    versions: bool,
}

#[system]
pub async fn list(args: ArgsRef<ListPluginsArgs>, proto: ResourceRef<ProtoResource>) {
    if !args.json {
        info!("Loading plugins...");
    }

    let mut config = proto.env.load_config()?.to_owned();

    let mut tools = proto
        .load_tools_with_filters(FxHashSet::from_iter(&args.ids))
        .await?;

    tools.sort_by(|a, d| a.id.cmp(&d.id));

    // --json
    if args.json {
        let items = tools
            .into_iter()
            .map(|t| {
                let tool_config = config.tools.get(&t.id);
                let name = t.get_name().to_owned();

                (
                    t.id,
                    PluginItem {
                        name,
                        locator: t.locator,
                        config: tool_config,
                        manifest: t.manifest,
                    },
                )
            })
            .collect::<FxHashMap<_, _>>();

        println!("{}", json::to_string_pretty(&items).into_diagnostic()?);

        return Ok(());
    }

    let printer = Mutex::new(Printer::new());
    let latest_version = UnresolvedVersionSpec::default();

    for tool in tools {
        let tool_config = config.tools.remove(&tool.id).unwrap_or_default();
        let inventory_dir = tool.get_inventory_dir();

        let mut versions = tool.load_version_resolver(&latest_version).await?;
        versions.aliases.extend(tool_config.aliases);

        let mut printer = printer.lock().await;

        printer.line();
        printer.header(&tool.id, &tool.metadata.name);

        printer.section(|p| {
            p.entry("Store", color::path(tool.get_inventory_dir()));

            if let Some(locator) = &tool.locator {
                p.locator(locator);
            }

            // --aliases
            if args.aliases {
                p.entry_map(
                    "Aliases",
                    versions
                        .aliases
                        .iter()
                        .map(|(k, v)| (color::hash(k), format_value(v.to_string())))
                        .collect::<Vec<_>>(),
                    None,
                );
            }

            // --versions
            if args.versions {
                let mut versions = tool.manifest.installed_versions.iter().collect::<Vec<_>>();
                versions.sort();

                p.entry_map(
                    "Versions",
                    versions
                        .iter()
                        .map(|version| {
                            let mut comments = vec![];
                            let mut is_default = false;

                            if let Some(meta) = &tool.manifest.versions.get(version) {
                                if let Some(at) = create_datetime(meta.installed_at) {
                                    comments.push(format!("installed {}", at.format("%x")));
                                }

                                if let Ok(Some(last_used)) = tool
                                    .manifest
                                    .load_used_at(inventory_dir.join(version.to_string()))
                                {
                                    if let Some(at) = create_datetime(last_used) {
                                        comments.push(format!("last used {}", at.format("%x")));
                                    }
                                }
                            }

                            if config
                                .versions
                                .get(&tool.id)
                                .is_some_and(|dv| *dv == version.to_unresolved_spec())
                            {
                                comments.push("default version".into());
                                is_default = true;
                            }

                            (
                                if is_default {
                                    color::invalid(version.to_string())
                                } else {
                                    color::hash(version.to_string())
                                },
                                format_value(comments.join(", ")),
                            )
                        })
                        .collect::<Vec<_>>(),
                    None,
                );
            }

            Ok(())
        })?;
    }

    printer.lock().await.flush();
}

fn create_datetime(millis: u128) -> Option<NaiveDateTime> {
    DateTime::from_timestamp((millis / 1000) as i64, ((millis % 1000) * 1_000_000) as u32)
        .map(|dt| dt.naive_local())
}
