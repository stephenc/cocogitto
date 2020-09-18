use anyhow::Result;
use clap::{App, AppSettings, Arg, SubCommand};
use cocogitto::changelog::WriterMode;
use cocogitto::commit::CommitType;
use cocogitto::filter::{CommitFilter, CommitFilters};
use cocogitto::version::VersionIncrement;
use cocogitto::CocoGitto;
use moins::Moins;
use std::process::exit;

const APP_SETTINGS: &[AppSettings] = &[
    AppSettings::SubcommandRequiredElseHelp,
    AppSettings::UnifiedHelpMessage,
    AppSettings::ColoredHelp,
    AppSettings::VersionlessSubcommands,
];

const SUBCOMMAND_SETTINGS: &[AppSettings] = &[
    AppSettings::UnifiedHelpMessage,
    AppSettings::ColoredHelp,
    AppSettings::VersionlessSubcommands,
];

const BUMP: &str = "bump";
const CHECK: &str = "check";
const LOG: &str = "log";
const VERIFY: &str = "verify";
const CHANGELOG: &str = "changelog";
const INIT: &str = "init";

fn main() -> Result<()> {
    let check_command = SubCommand::with_name(CHECK)
        .settings(SUBCOMMAND_SETTINGS)
        .about("Verify all commit message against the conventional commit specification")
        .arg(
            Arg::with_name("edit")
                .help("Interactively rename invalid commit message")
                .short("e")
                .long("edit"),
        )
        .display_order(1);

    let log_command = SubCommand::with_name(LOG)
        .settings(SUBCOMMAND_SETTINGS)
        .about("Like git log but for conventional commits")
        .arg(
            Arg::with_name("breaking-change")
                .help("filter BREAKING CHANGE commit")
                .short("B")
                .long("breaking-change"),
        )
        .arg(
            Arg::with_name("type")
                .help("filter on commit type")
                .short("t")
                .takes_value(true)
                .multiple(true)
                .long("type"),
        )
        .arg(
            Arg::with_name("author")
                .help("filter on commit author")
                .short("a")
                .takes_value(true)
                .multiple(true)
                .long("author"),
        )
        .arg(
            Arg::with_name("scope")
                .help("filter on commit scope")
                .short("s")
                .takes_value(true)
                .multiple(true)
                .long("scope"),
        )
        .arg(
            Arg::with_name("no-error")
                .help("omit error on the commit log")
                .short("e")
                .long("no-error"),
        )
        .display_order(2);

    let verify_command = SubCommand::with_name(VERIFY)
        .settings(SUBCOMMAND_SETTINGS)
        .about("Verify a single commit message")
        .arg(Arg::with_name("message").help("The commit message"))
        .display_order(3);
    let changelog_command = SubCommand::with_name(CHANGELOG)
        .settings(SUBCOMMAND_SETTINGS)
        .about("Display a changelog for a given commit oid range")
        .arg(
            Arg::with_name("from")
                .help("Generate the changelog from this commit or tag ref, default latest tag")
                .short("f")
                .long("from")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("to")
                .help("Generate the changelog to this commit or tag ref, default HEAD")
                .short("t")
                .long("to")
                .takes_value(true),
        )
        .display_order(4);

    let bump_command = SubCommand::with_name(BUMP)
        .settings(SUBCOMMAND_SETTINGS)
        .about("Commit changelog from latest tag to HEAD and create a new tag")
        .arg(
            Arg::with_name("version")
                .help("Manually set the next version")
                .short("v")
                .takes_value(true)
                .long("version")
                .required_unless_one(&["auto", "major", "patch", "minor"]),
        )
        .arg(
            Arg::with_name("auto")
                .help("Atomatically suggest the next version")
                .short("a")
                .long("auto")
                .required_unless_one(&["version", "major", "patch", "minor"]),
        )
        .arg(
            Arg::with_name("major")
                .help("Increment the major version")
                .short("M")
                .long("major")
                .required_unless_one(&["version", "auto", "patch", "minor"]),
        )
        .arg(
            Arg::with_name("patch")
                .help("Increment the patch version")
                .short("p")
                .long("patch")
                .required_unless_one(&["version", "auto", "major", "minor"]),
        )
        .arg(
            Arg::with_name("minor")
                .help("Increment the minor version")
                .short("m")
                .long("minor")
                .required_unless_one(&["version", "auto", "patch", "major"]),
        )
        .display_order(5);

    let init_subcommand = SubCommand::with_name(INIT)
        .settings(SUBCOMMAND_SETTINGS)
        .about("Install cog config files");

    let on_configured_repo_subcommands =
        vec![check_command, log_command, changelog_command, bump_command];

    let default_subcommands = vec![verify_command, init_subcommand];

    let mut app =  App::new("Coco Gitto")
        .settings(APP_SETTINGS)
        .version(env!("CARGO_PKG_VERSION"))
        .author("Paul D. <paul.delafosse@protonmail.com>")
        .about("A conventional commit compliant, changelog and commit generator")
        .long_about("Conventional Commit Git Terminal Overlord is a tool to help you use the conventional commit specification")
        .subcommands(default_subcommands);

    let cocogitto = CocoGitto::get();

    if cocogitto.is_ok() {
        let commit_subcommands = CocoGitto::get_commit_metadata()
            .iter()
            .map(|(commit_type, commit_config)| {
                SubCommand::with_name(commit_type.get_key_str())
                    .settings(SUBCOMMAND_SETTINGS)
                    .about(commit_config.help_message.as_str())
                    .help(commit_config.help_message.as_str())
                    .arg(Arg::with_name("message").help("The commit message"))
                    .arg(Arg::with_name("scope").help("The scope of the commit message"))
                    .arg(Arg::with_name("body").help("The body of the commit message"))
                    .arg(Arg::with_name("footer").help("The footer of the commit message"))
                    .arg(
                        Arg::with_name("breaking-change")
                            .help("BREAKING CHANGE commit")
                            .short("B")
                            .long("breaking-change"),
                    )
            })
            .collect::<Vec<App>>();

        app = app
            .subcommands(on_configured_repo_subcommands)
            .subcommands(commit_subcommands)
            .display_order(6);
    };

    let matches = app.get_matches();

    if let Some(subcommand) = matches.subcommand_name() {
        match subcommand {
            BUMP => {
                let subcommand = matches.subcommand_matches(BUMP).unwrap();

                let increment = if let Some(version) = subcommand.value_of("version") {
                    VersionIncrement::Manual(version.to_string())
                } else if subcommand.is_present("auto") {
                    VersionIncrement::Auto
                } else if subcommand.is_present("major") {
                    VersionIncrement::Major
                } else if subcommand.is_present("patch") {
                    VersionIncrement::Patch
                } else if subcommand.is_present("minor") {
                    VersionIncrement::Minor
                } else {
                    unreachable!()
                };

                // TODO mode to cli
                cocogitto?.create_version(increment, WriterMode::Prepend)?
            }
            VERIFY => {
                let subcommand = matches.subcommand_matches(VERIFY).unwrap();
                let message = subcommand.value_of("message").unwrap();

                match cocogitto?.verify(message) {
                    Ok(()) => exit(0),
                    Err(err) => {
                        eprintln!("{}", err);
                        exit(1);
                    }
                }
            }

            CHECK => {
                let subcommand = matches.subcommand_matches(CHECK).unwrap();
                if subcommand.is_present("edit") {
                    cocogitto?.check_and_edit()?;
                } else {
                    cocogitto?.check()?
                }
            }
            LOG => {
                let subcommand = matches.subcommand_matches(LOG).unwrap();

                let mut filters = vec![];
                if let Some(commit_types) = subcommand.values_of("type") {
                    commit_types.for_each(|commit_type| {
                        filters.push(CommitFilter::Type(CommitType::from(commit_type)));
                    });
                }

                if let Some(scopes) = subcommand.values_of("scope") {
                    scopes.for_each(|scope| {
                        filters.push(CommitFilter::Scope(scope.to_string()));
                    });
                }

                if let Some(authors) = subcommand.values_of("author") {
                    authors.for_each(|author| {
                        filters.push(CommitFilter::Author(author.to_string()));
                    });
                }

                if subcommand.is_present("breaking-change") {
                    filters.push(CommitFilter::BreakingChange);
                }

                if subcommand.is_present("no-error") {
                    filters.push(CommitFilter::NoError);
                }

                let filters = CommitFilters(filters);

                let mut content = cocogitto?.get_log(filters)?;
                Moins::run(&mut content, None);
            }
            CHANGELOG => {
                let subcommand = matches.subcommand_matches(CHANGELOG).unwrap();
                let from = subcommand.value_of("from");
                let to = subcommand.value_of("to");
                let result = cocogitto?.get_colored_changelog(from, to)?;
                println!("{}", result);
            }

            INIT => {
                let _subcommand = matches.subcommand_matches(INIT).unwrap();
                cocogitto::init()?;
            }

            commit_type => {
                if let Some(args) = matches.subcommand_matches(commit_type) {
                    let message = args.value_of("message").unwrap().to_string();
                    let scope = args.value_of("scope").map(|scope| scope.to_string());
                    let body = args.value_of("body").map(|body| body.to_string());
                    let footer = args.value_of("footer").map(|footer| footer.to_string());
                    let breaking_change = args.is_present("breaking-change");
                    cocogitto?.conventional_commit(
                        commit_type,
                        scope,
                        message,
                        body,
                        footer,
                        breaking_change,
                    )?;
                }
            }
        }
    }
    Ok(())
}
