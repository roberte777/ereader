//! E-reader CLI tool for administration.

use clap::{Parser, Subcommand};
use common::config::AppConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "ereader-cli")]
#[command(author, version, about = "E-reader administration CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// User management commands
    User {
        #[command(subcommand)]
        action: UserCommands,
    },
    /// Book management commands
    Book {
        #[command(subcommand)]
        action: BookCommands,
    },
    /// Database migration commands
    Migrate {
        #[command(subcommand)]
        action: MigrateCommands,
    },
    /// Health check
    Health,
}

#[derive(Subcommand)]
enum UserCommands {
    /// List all users
    List,
    /// Show user details
    Show {
        #[arg(help = "User ID")]
        id: String,
    },
}

#[derive(Subcommand)]
enum BookCommands {
    /// List books for a user
    List {
        #[arg(help = "User ID")]
        user_id: String,
        #[arg(short, long, default_value = "20")]
        limit: i64,
    },
    /// Import a book from a file
    Import {
        #[arg(help = "User ID")]
        user_id: String,
        #[arg(help = "Path to the book file")]
        file_path: String,
    },
    /// Delete a book
    Delete {
        #[arg(help = "Book ID")]
        id: String,
    },
    /// Show book details
    Show {
        #[arg(help = "Book ID")]
        id: String,
    },
}

#[derive(Subcommand)]
enum MigrateCommands {
    /// Run pending migrations
    Run,
    /// Show migration status
    Status,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ereader_cli=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();
    let config = AppConfig::load()?;

    match cli.command {
        Commands::User { action } => handle_user_command(action, &config).await,
        Commands::Book { action } => handle_book_command(action, &config).await,
        Commands::Migrate { action } => handle_migrate_command(action, &config).await,
        Commands::Health => handle_health(&config).await,
    }
}

async fn handle_user_command(action: UserCommands, config: &AppConfig) -> anyhow::Result<()> {
    let pool = db_layer::create_pool(&config.database).await?;

    match action {
        UserCommands::List => {
            let users = db_layer::queries::UserQueries::list_all(&pool).await?;
            println!("Users ({}):", users.len());
            for user in users {
                println!("  {} - {}", user.id, user.email.as_deref().unwrap_or("(no email)"));
            }
        }
        UserCommands::Show { id } => {
            let user = db_layer::queries::UserQueries::get_by_id(&pool, &id).await?;
            match user {
                Some(u) => {
                    println!("User: {}", u.id);
                    println!("  Email: {}", u.email.as_deref().unwrap_or("(none)"));
                    println!("  Created: {}", u.created_at);
                    println!("  Updated: {}", u.updated_at);
                }
                None => {
                    println!("User not found: {}", id);
                }
            }
        }
    }

    Ok(())
}

async fn handle_book_command(action: BookCommands, config: &AppConfig) -> anyhow::Result<()> {
    let pool = db_layer::create_pool(&config.database).await?;

    match action {
        BookCommands::List { user_id, limit } => {
            let pagination = common::Pagination {
                limit,
                offset: 0,
            };
            let sort = db_layer::queries::BookSortOptions::by_created_at_desc();
            let filter = db_layer::queries::BookFilterOptions::default();
            let books = db_layer::queries::BookQueries::list_for_user(&pool, &user_id, &pagination, &sort, &filter).await?;

            println!("Books for user {} ({} total):", user_id, books.total);
            for book in books.items {
                println!("  {} - {} by {}", book.id, book.title, book.authors.join(", "));
            }
        }
        BookCommands::Show { id } => {
            let book_id = uuid::Uuid::parse_str(&id)?;
            let book = db_layer::queries::BookQueries::get_by_id(&pool, book_id).await?;
            match book {
                Some(b) => {
                    println!("Book: {}", b.id);
                    println!("  Title: {}", b.title);
                    println!("  Authors: {}", b.authors.join(", "));
                    println!("  Description: {}", b.description.as_deref().unwrap_or("(none)"));
                    println!("  Language: {}", b.language.as_deref().unwrap_or("(none)"));
                    println!("  Publisher: {}", b.publisher.as_deref().unwrap_or("(none)"));
                    println!("  ISBN: {}", b.isbn.as_deref().unwrap_or("(none)"));
                    println!("  Created: {}", b.created_at);
                    println!("  Updated: {}", b.updated_at);
                }
                None => {
                    println!("Book not found: {}", id);
                }
            }
        }
        BookCommands::Import { user_id, file_path } => {
            println!("Importing book from: {}", file_path);

            // Read the file
            let data = tokio::fs::read(&file_path).await?;

            // Determine format from extension
            let format = if file_path.ends_with(".epub") {
                common::BookFormat::Epub
            } else if file_path.ends_with(".pdf") {
                common::BookFormat::Pdf
            } else {
                anyhow::bail!("Unsupported file format. Only .epub and .pdf are supported.");
            };

            // Get handler and extract metadata
            let handler = indexer::handler_for_format(format)
                .ok_or_else(|| anyhow::anyhow!("No handler for format"))?;
            let metadata = handler.extract_metadata(&data)?;

            // Compute hash
            let hash = storage_layer::LocalStorage::compute_hash(&data);

            // Create book
            let create_book = db_layer::models::CreateBook::new(&user_id, metadata.title.unwrap_or_else(|| "Unknown".to_string()))
                .with_authors(metadata.authors);

            let book = db_layer::queries::BookQueries::create(&pool, &create_book).await?;

            // Store the file
            let storage = storage_layer::LocalStorage::from_config(&config.storage).await?;
            let storage_path = storage_layer::traits::Storage::store(&storage, &hash, &data).await?;

            // Create file asset
            let file_name = std::path::Path::new(&file_path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            let file_asset = db_layer::models::CreateFileAsset::new(
                book.id,
                format,
                data.len() as i64,
                hash.as_str(),
                &storage_path,
                file_name,
            );

            db_layer::queries::FileAssetQueries::create(&pool, &file_asset).await?;

            println!("Book imported successfully!");
            println!("  ID: {}", book.id);
            println!("  Title: {}", book.title);
        }
        BookCommands::Delete { id } => {
            let book_id = uuid::Uuid::parse_str(&id)?;
            let deleted = db_layer::queries::BookQueries::delete(&pool, book_id).await?;
            if deleted {
                println!("Book deleted: {}", id);
            } else {
                println!("Book not found: {}", id);
            }
        }
    }

    Ok(())
}

async fn handle_migrate_command(action: MigrateCommands, config: &AppConfig) -> anyhow::Result<()> {
    let pool = db_layer::create_pool(&config.database).await?;

    match action {
        MigrateCommands::Run => {
            println!("Running migrations...");
            db_layer::run_migrations(&pool).await?;
            println!("Migrations completed successfully.");
        }
        MigrateCommands::Status => {
            println!("Checking migration status...");
            // SQLx doesn't expose a simple way to check status,
            // so we just verify connection and migrations table exists
            let connected = db_layer::health_check(&pool).await;
            if connected {
                println!("Database connected and migrations table exists.");
            } else {
                println!("Database connection failed.");
            }
        }
    }

    Ok(())
}

async fn handle_health(config: &AppConfig) -> anyhow::Result<()> {
    println!("Checking health...");

    // Check database
    print!("  Database: ");
    match db_layer::create_pool(&config.database).await {
        Ok(pool) => {
            if db_layer::health_check(&pool).await {
                println!("OK");
            } else {
                println!("UNHEALTHY");
            }
        }
        Err(e) => {
            println!("ERROR - {}", e);
        }
    }

    // Check storage
    print!("  Storage: ");
    match storage_layer::LocalStorage::from_config(&config.storage).await {
        Ok(storage) => {
            if storage.health_check().await {
                println!("OK");
            } else {
                println!("UNHEALTHY");
            }
        }
        Err(e) => {
            println!("ERROR - {}", e);
        }
    }

    Ok(())
}
