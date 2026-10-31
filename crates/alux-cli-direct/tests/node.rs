//! Runs a single-binary program configured from options and the environment, as a node is.

use alux_cli::{CliCommandsAlg, CliProgramBuilder, cli};
use alux_cli_direct::{CliDirect, Exit};
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
    }

    /// Declares the same command line in A-normal form: each part bound once, then named.
    fn node_anf_cli<Machine>(&self) {
        let dev = self.named("Development mode").switch().env("NEXTEST");
        let rpc_addr = self.named("RPC address").optional();
        let peers = self.named("Peers to dial").many().env("CARGO_PKG_VERSION").delimiter('.');
        let run = self.op(Machine::run_node).arg("dev", dev).arg("rpc_addr", rpc_addr).arg("peers", peers);
        let check = self.op(Machine::check).arg("file", self.positional("Configuration file"));
        self.commands().cmd_root(run).cmd("check", check)
    }
}

struct Local;

impl Node for Local {
    fn name(&self) -> String {
        "local".to_owned()
    }
}

fn node(line: &str) -> Exit {
    CliDirect::new(Arc::new(Local))
        .version("1.2.3")
        .run_blocking(CliProgramBuilder.node_cli::<Local>(), line.split_whitespace())
}

#[test]
fn runs_the_programs_own_command_with_no_command_word() {
    // nextest sets `NEXTEST=1` and Cargo sets `CARGO_PKG_VERSION` for every test it runs, so the
    // switch is on and the peers are this package's version split at its dots.
    let peers = env!("CARGO_PKG_VERSION").split('.').collect::<Vec<_>>();
    assert_eq!(node("node").stdout, format!("local dev=true rpc=None peers={peers:?}\n"));
    // The command line wins over the environment, and each of its words is split too.
    assert_eq!(
        node("node --dev --rpc-addr 127.0.0.1 --peers a.b --peers c").stdout,
        "local dev=true rpc=Some(127.0.0.1) peers=[\"a\", \"b\", \"c\"]\n"
    );
    // A named command is selected before the program's own.
    assert_eq!(node("node check node.toml").stdout, "checked node.toml\n");
}

#[test]
fn answers_version_and_helps_the_programs_own_command_with_the_others() {
    assert_eq!(node("node --version"), Exit { status: 0, stdout: "node 1.2.3\n".into(), stderr: String::new() });
    let help = node("node --help").stdout;
    assert!(help.starts_with("Usage: node [options]\n       node <command>\n"), "{help}");
    assert!(help.contains("check") && help.contains("--version"), "{help}");
    // The switch lists the variable it falls back to, and the list its delimiter.
    let line = |spelling: &str| help.lines().find(|line| line.contains(spelling)).unwrap_or_default().to_owned();
    assert!(line("--dev").ends_with("Development mode [env: NEXTEST]"), "{help}");
    // The delimiter here is `.`, so the list reads `<peers>.` followed by `...`.
    assert!(line("--peers").contains("--peers <peers>...."), "{help}");
    // A value is named as the command line spells it.
    assert!(line("--rpc-addr").contains("--rpc-addr <rpc-addr>  RPC address"), "{help}");
}

#[test]
fn reads_a_declaration_in_a_normal_form_as_its_nested_form() {
    let anf = |line: &str| {
        CliDirect::new(Arc::new(Local))
            .version("1.2.3")
            .run_blocking(CliProgramBuilder.node_anf_cli::<Local>(), line.split_whitespace())
    };
    for line in
        ["node", "node --dev --rpc-addr 127.0.0.1 --peers a.b", "node check node.toml", "node --help", "node --nope"]
    {
        assert_eq!(anf(line), node(line), "{line}");
    }
}
