//! Runs a single-binary program configured from options and the environment through `clap`.

use alux_cli::{CliCommandsAlg, CliProgramBuilder, cli};
use alux_cli_clap::CliClap;
use alux_cli_direct::Exit;
use alux_ext::ext;
use std::{net::IpAddr, sync::Arc};

trait Node {
    fn name(&self) -> String;
}

#[ext(name = NodeExt, defunc)]
impl<This> This
where
    This: Node,
{
    /// Runs the node.
    fn run_node(&self, dev: bool, rpc_addr: Option<IpAddr>, peers: Vec<String>) -> String {
        format!("{} dev={dev} rpc={rpc_addr:?} peers={peers:?}", self.name())
    }

    /// Checks the configuration.
    fn check(&self, file: String) -> String {
        format!("checked {file}")
    }

    /// Reports whether it is quiet.
    fn quiet(&self, quiet: bool) -> String {
        format!("quiet={quiet}")
    }
}

#[ext(name = NodeCliExt, defunc(via = cli))]
impl<This> This
where
    This: CliCommandsAlg,
{
    fn node_cli<Machine>(&self) {
        self.commands()
            .cmd_root(
                self.op(Machine::run_node)
                    .arg("dev", self.named("Development mode").switch().env("NEXTEST"))
                    .arg("rpc_addr", self.named("RPC address").optional())
                    .arg("peers", self.named("Peers to dial").many().env("CARGO_PKG_VERSION").delimiter('.')),
            )
            .cmd("check", self.op(Machine::check).arg("file", self.positional("Configuration file")))
            .cmd(
                "quiet",
                self.op(Machine::quiet).arg("quiet", self.named("Quiet").switch().env("CARGO_PKG_VERSION_PRE")),
            )
    }
}

struct Local;

impl Node for Local {
    fn name(&self) -> String {
        "local".to_owned()
    }
}

fn node(line: &str) -> Exit {
    CliClap::new(Arc::new(Local), CliProgramBuilder.node_cli::<Local>())
        .version("1.2.3")
        .run_blocking(line.split_whitespace())
}

#[test]
fn reads_a_switch_and_a_list_from_the_environment_unless_the_command_line_gives_them() {
    // nextest sets `NEXTEST=1` and Cargo sets `CARGO_PKG_VERSION` for every test it runs, so the
    // switch is on and the peers are this package's version split at its dots.
    let peers = env!("CARGO_PKG_VERSION").split('.').collect::<Vec<_>>();
    let bare = node("node");
    assert_eq!(bare.stdout, format!("local dev=true rpc=None peers={peers:?}\n"), "{bare:?}");
    assert_eq!(
        node("node --rpc-addr 127.0.0.1 --peers a.b --peers c").stdout,
        "local dev=true rpc=Some(127.0.0.1) peers=[\"a\", \"b\", \"c\"]\n"
    );
    assert_eq!(node("node check node.toml").stdout, "checked node.toml\n");
}

#[test]
fn answers_version_as_clap_does() {
    assert_eq!(node("node --version"), Exit { status: 0, stdout: "node 1.2.3\n".into(), stderr: String::new() });
    assert_eq!(node("node -V").stdout, "node 1.2.3\n");
    let help = node("node --help").stdout;
    assert!(help.contains("Development mode [env: NEXTEST]"), "{help}");
}

#[test]
fn turns_a_switch_off_as_its_spelling_or_an_empty_variable_says() {
    let peers = env!("CARGO_PKG_VERSION").split('.').collect::<Vec<_>>();
    let dev = |line: &str| node(line).stdout.replace(&format!(" rpc=None peers={peers:?}\n"), "");
    // A long spelling states a truth, in any case, over `NEXTEST=1`.
    assert_eq!(dev("node --dev=false"), "local dev=false");
    assert_eq!(dev("node --dev=OFF"), "local dev=false");
    assert_eq!(dev("node --dev=0"), "local dev=false");
    assert_eq!(dev("node --dev=On"), "local dev=true");
    assert_eq!(dev("node --dev"), "local dev=true");
    assert_eq!(node("node --dev=maybe").status, 2);
    // Cargo sets `CARGO_PKG_VERSION_PRE` to nothing for a release, which states nothing.
    assert_eq!(node("node quiet").stdout, "quiet=false\n");
    assert_eq!(node("node quiet --quiet").stdout, "quiet=true\n");
}
