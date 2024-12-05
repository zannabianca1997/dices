use std::{env, error::Error, fs, iter::once, path::PathBuf};

use dices_server_migration::{
    cli::run_migrate,
    sea_orm::{ConnectOptions, Database},
    Migrator,
};
use sea_orm_cli::{run_generate_command, MigrateSubcommands};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt, TestcontainersError},
};
use walkdir::WalkDir;

async fn db() -> Result<ContainerAsync<Postgres>, TestcontainersError> {
    Postgres::default()
        .with_db_name("dices_server_build")
        .with_user("dices_server_build")
        .with_password("dices_server_build")
        .with_tag("17.0-alpine3.20")
        .start()
        .await
}

const VERBOSE: bool = true;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    WalkDir::new("./migration/src")
        .into_iter()
        .filter_map(Result::ok)
        .map(|i| i.path().to_path_buf())
        .chain(once(PathBuf::from("./migration/Cargo.toml")))
        .for_each(|i| cargo_emit::rerun_if_changed!(i.display()));

    // building an example db
    let db = db().await?;

    // connecting to it
    let url = format!(
        "postgres://dices_server_build:dices_server_build@{}:{}/dices_server_build",
        db.get_host().await?,
        db.get_host_port_ipv4(5432).await?
    );
    let connect_options = ConnectOptions::new(&url)
        .set_schema_search_path("public")
        .to_owned();
    let conn = Database::connect(connect_options).await?;

    // run all migration, as if the app is starting
    run_migrate(
        Migrator,
        &conn,
        Some(MigrateSubcommands::Up { num: None }),
        VERBOSE,
    )
    .await?;

    conn.close().await?;

    // generate entities from the filled database
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("./generated/entities/");
    run_generate_command(
        sea_orm_cli::GenerateSubcommands::Entity {
            compact_format: false,
            expanded_format: false,
            include_hidden_tables: false,
            tables: vec![],
            ignore_tables: vec!["seaql_migrations".to_owned()],
            max_connections: 1,
            output_dir: out_dir.to_string_lossy().into_owned(),
            database_schema: Some("public".to_owned()),
            database_url: url,
            with_serde: "none".to_owned(),
            serde_skip_deserializing_primary_key: false,
            serde_skip_hidden_column: false,
            with_copy_enums: true,
            date_time_crate: sea_orm_cli::DateTimeCrate::Chrono,
            lib: false,
            model_extra_derives: vec![],
            model_extra_attributes: vec![],
            enum_extra_derives: vec![],
            enum_extra_attributes: vec![],
            seaography: false,
        },
        VERBOSE,
    )
    .await?;

    let attr_file = out_dir.join("../entities_attr.rs");
    fs::write(
        &attr_file,
        format!(
            "#[path = \"{}\"]\n#[allow(unused_imports)]\nmod entities;",
            out_dir.join("mod.rs").display()
        ),
    )?;

    cargo_emit::rustc_env!("ENTITY_MODULE", "{}", attr_file.display());

    // Destroy the database
    db.stop().await?;
    db.rm().await?;

    Ok(())
}
