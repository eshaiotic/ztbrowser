//! The commands the Tor window sends to its Tor.

use ztbrowser_core::tor::control_command::ControlCommand;

struct ControlCommandScenario {
    command: Option<ControlCommand>,
}

impl ControlCommandScenario {
    fn given_command(command: ControlCommand) -> Self {
        Self { command: Some(command) }
    }

    fn given_cookie(cookie: &[u8]) -> Self {
        Self { command: ControlCommand::authenticate(cookie) }
    }

    fn when_sent(&self) -> CommandOutcome {
        CommandOutcome { line: self.command.as_ref().map(ControlCommand::to_line) }
    }
}

struct CommandOutcome {
    line: Option<String>,
}

impl CommandOutcome {
    fn then_sends(&self, line: &str) {
        assert_eq!(self.line.as_deref(), Some(line));
    }

    fn then_sends_nothing(&self) {
        assert_eq!(self.line, None);
    }
}

#[test]
fn should_prove_access_with_the_cookie_in_hexadecimal() {
    let cookie: Vec<u8> = (0..32).collect();

    ControlCommandScenario::given_cookie(&cookie)
        .when_sent()
        .then_sends("AUTHENTICATE 000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F\r\n");
}

#[test]
fn should_not_authenticate_with_a_truncated_cookie() {
    ControlCommandScenario::given_cookie(&[1, 2, 3]).when_sent().then_sends_nothing();
}

#[test]
fn should_ask_tor_for_new_circuits_on_new_identity() {
    ControlCommandScenario::given_command(ControlCommand::NewIdentity).when_sent().then_sends("SIGNAL NEWNYM\r\n");
}

#[test]
fn should_ask_where_tor_listens_for_socks() {
    ControlCommandScenario::given_command(ControlCommand::GetSocksListeners)
        .when_sent()
        .then_sends("GETINFO net/listeners/socks\r\n");
}
