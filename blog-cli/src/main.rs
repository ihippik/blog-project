use std::fs;
use std::io::{self, Write};
use std::path::Path;

use anyhow::Result;
use clap::{Parser, Subcommand};
use blog_client::{BlogClient, Transport};
use blog_client::error::BlogClientError;
use uuid::Uuid;

const TOKEN_FILE: &str = ".blog_token";

#[derive(Parser, Debug)]
#[command(name = "blog-cli")]
#[command(about = "CLI client for the blog server (HTTP/gRPC)", long_about = None)]
struct Cli {
    #[arg(long)]
    grpc: bool,

    #[arg(long)]
    server: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Register {
        #[arg(long)]
        username: String,
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
    },
    Login {
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
    },
    Create {
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: String,
    },
    Get {
        #[arg(long)]
        id: String,
    },
    Update {
        #[arg(long)]
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        content: Option<String>,
    },
    Delete {
        #[arg(long)]
        id: String,
    },
    List {
        #[arg(long, default_value_t = 20)]
        limit: u32,
        #[arg(long, default_value_t = 0)]
        offset: u32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    let server_addr = cli.server.unwrap_or_else(|| {
        if cli.grpc {
            "http://127.0.0.1:50051".to_string()
        } else {
            "http://127.0.0.1:8080".to_string()
        }
    });

    let transport = if cli.grpc {
        Transport::Grpc(server_addr)
    } else {
        Transport::Http(server_addr)
    };

    let mut client = BlogClient::new(transport)
        .await
        .map_err(map_client_err)?;

    if let Some(token) = load_token() {
        client.set_token(token);
    }

    match cli.command {
        Commands::Register { username, email, password } => {
            let resp = client
                .register(username.clone(), email, password)
                .await
                .map_err(map_client_err)?;

            if let Some(token) = resp.access_token.as_ref() {
                save_token(token)?;
                println!("✅ User registered, token saved.");
            } else {
                println!("⚠ Registration succeeded, but no token was returned.");
            }

            if let Some(user) = resp.user {
                println!("user: {} <{}>", user.username, user.email);
            }
        }

        Commands::Login { email, password } => {
            let resp = client
                .login(email.clone(), password)
                .await
                .map_err(map_client_err)?;

            if let Some(token) = resp.access_token.as_ref() {
                save_token(token)?;
                println!("✅ Login successful, token saved.");
            } else {
                println!("⚠ Login succeeded, but no token was returned.");
            }

            if let Some(user) = resp.user {
                println!("user: {} <{}>", user.username, user.email);
            }
        }

        Commands::Create { title, content } => {
            let post = client
                .create_post(title, content)
                .await
                .map_err(map_client_err)?;

            println!("✅ Post created:");
            print_post(&post);
        }

        Commands::Get { id } => {
            let id = parse_uuid(&id)?;
            let post = client
                .get_post(id)
                .await
                .map_err(map_client_err)?;

            print_post(&post);
        }

        Commands::Update { id, title, content } => {
            let id = parse_uuid(&id)?;
            let mut post = client
                .get_post(id)
                .await
                .map_err(map_client_err)?;

            if let Some(t) = title {
                post.title = t;
            }
            if let Some(c) = content {
                post.content = c;
            }

            let updated = client
                .update_post(post.id, post.title.clone(), post.content.clone())
                .await
                .map_err(map_client_err)?;

            println!("✅ Post updated:");
            print_post(&updated);
        }

        Commands::Delete { id } => {
            let id = parse_uuid(&id)?;
            client
                .delete_post(id)
                .await
                .map_err(map_client_err)?;

            println!("🗑 Post deleted.");
        }

        Commands::List { limit, offset } => {
            let posts = client
                .list_posts(limit, offset)
                .await
                .map_err(map_client_err)?;

            if posts.is_empty() {
                println!("(there are no posts yet)");
            } else {
                for p in posts {
                    println!("------------------------------");
                    print_post(&p);
                }
            }
        }
    }

    Ok(())
}

fn parse_uuid(input: &str) -> Result<Uuid> {
    Ok(Uuid::parse_str(input)?)
}

fn load_token() -> Option<String> {
    if !Path::new(TOKEN_FILE).exists() {
        return None;
    }

    match fs::read_to_string(TOKEN_FILE) {
        Ok(s) => {
            let t = s.trim().to_string();
            if t.is_empty() {
                None
            } else {
                Some(t)
            }
        }
        Err(_) => None,
    }
}

fn save_token(token: &str) -> io::Result<()> {
    let mut file = fs::File::create(TOKEN_FILE)?;
    file.write_all(token.as_bytes())?;
    Ok(())
}

fn print_post(post: &blog_client::models::Post) {
    println!("id:         {}", post.id);
    println!("title:      {}", post.title);
    println!("content:    {}", post.content);
    println!("author_id:  {}", post.author_id);
    println!("created_at: {}", post.created_at);

    if let Some(updated_at) = post.updated_at {
        println!("updated_at: {}", updated_at);
    }
}

fn map_client_err(err: BlogClientError) -> anyhow::Error {
    anyhow::anyhow!(err)
}
