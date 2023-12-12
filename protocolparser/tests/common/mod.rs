use testcontainers::{clients::Cli, Container};
use testcontainers_modules::mysql::Mysql;

pub fn setup_mysql<'a>(client: &'a Cli) -> Container<'a, Mysql> {
    client.run(Mysql::default())
}
