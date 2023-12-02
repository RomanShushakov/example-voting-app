use std::convert::Infallible;
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use tokio_postgres::{NoTls, Error, types::ToSql, Row};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, StatusCode, Server};
use std::sync::{RwLock, Mutex, Arc};
use std::collections::LinkedList;


fn get_db_url() -> String 
{
    if let Ok(url) = std::env::var("DATABASE_URL") 
    {
        url
    } 
    else 
    {
        "postgres://postgres:postgres@localhost/postgres".into()
    }
}


struct Connection  
{
    client : tokio_postgres::Client,
}


impl Connection 
{
    fn new(client :tokio_postgres::Client) -> Connection 
    {
        Connection { client }
    }
}


#[derive(Default)]
pub struct Database
{
    pool : Mutex<RwLock<LinkedList<Connection>>>,
}


impl Database 
{
    pub fn new() -> Database 
    {
        Database 
        {
            pool : Mutex::new(RwLock::new(LinkedList::new())),
        }
    }

    async fn open_connection(&self) -> Result<Connection, Box<dyn std::error::Error>> 
    {
        // Connect to the database.
        let (client, connection) = tokio_postgres::connect(
            &*get_db_url(), NoTls,
        ).await?;

        tokio::spawn(async move 
            {
                if let Err(e) = connection.await 
                {
                    eprintln!("The connection was not set up properly: {}", e);
                }
            });
        Ok(Connection::new(client))
    }

    async fn get_connection(&self) -> Option<Connection> 
    {
        let connection : Option<Connection>;
        {
            let pool = self.pool.lock().unwrap();
            let mut pool = pool.write().unwrap();
            println!("Database pool length: {}", pool.len());
            connection = pool.pop_front();
        }
        match connection 
        {
            Some(connection) => Some(connection),
            None => 
            {
                match self.open_connection().await 
                {
                    Ok(connection) => Some(connection),
                    Err(_) => None,
                }
            },
        }
    }


    fn return_connection(&self, connection: Connection) 
    {
        let pool = self.pool.lock().unwrap();
        let mut pool = pool.write().unwrap();
        pool.push_front(connection);
    }

    
    pub async fn query<'a>(&self, query: &'a str, params: &'a[&'a(dyn ToSql + Sync)]) -> Option<Vec<Row>>  
    {
        println!("processing query");
        match self.get_connection().await 
        {
            Some(connection) => 
            {
                match connection.client.query(query, params).await 
                {
                    Ok(rows) => 
                    {
                        self.return_connection(connection);
                        Some(rows)
                    },
                    Err(e) => 
                    {
                        self.return_connection(connection);
                        eprintln!("{}", e);
                        None
                    },
                }
            },
            None => 
            {
                eprintln!("Failed to execute query {}", query);
                None
            }
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
struct Vote 
{
    vote: String,
    count: i64,
}


// CORS headers
fn response_build(body: &str) -> Response<Body> 
{
    Response::builder()
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        .header("Access-Control-Allow-Headers", "api,Keep-Alive,User-Agent,Content-Type")
        .body(Body::from(body.to_owned()))
        .unwrap()
}


async fn handle_request(req: Request<Body>, db: Arc<Database>) -> Result<Response<Body>, anyhow::Error> 
{
    match (req.method(), req.uri().path()) 
    {
        (&Method::GET, "/") => Ok(Response::new(Body::from(
            "The valid endpoints are /echo /votes",
        ))),

        // Simply echo the body back to the client.
        (&Method::POST, "/echo") => Ok(Response::new(req.into_body())),

        // CORS OPTIONS
        (&Method::OPTIONS, "/votes") => Ok(response_build(&String::from(""))),


        (&Method::GET, "/votes") => 
        {
            let query = "SELECT vote, COUNT(id) AS count FROM votes GROUP BY vote;";
            let params = &[];

            let votes = db.query(query, params).await.unwrap()
                .into_iter()
                .map(|row| Vote { vote: row.get(0), count: row.get(1) })
                .collect::<Vec<Vote>>();


            Ok(response_build(serde_json::to_string(&votes)?.as_str()))
        }        
        
        // Return the 404 Not Found for other routes.
        _ => 
        {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}


#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> 
{
    let database = Arc::new(Database::new());

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let make_svc = make_service_fn(|_| 
        {
            let database = database.clone();
            async move 
                {
                    
                    Ok::<_, Infallible>(service_fn(move |req| 
                        {
                            let db = database.clone();
                            handle_request(req, db)
                        }))
                }
        });
    let server = Server::bind(&addr).serve(make_svc);
    if let Err(e) = server.await 
    {
        eprintln!("server error: {}", e);
    }

    Ok(())
}
