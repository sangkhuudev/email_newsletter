use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            &self.username,&self.password,&self.host,&self.port,&self.database_name
        )
    }
}
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Initialise the configuration reader
    let mut settings = config::Config::default();

    // Add configuration file with name 'configuration' with extension: yaml,json...
    settings.merge(config::File::with_name("configuration"))?;

    // Try to convert into struct Settings
    settings.try_into()

}