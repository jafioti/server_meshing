use std::{sync::{RwLock, Arc}, thread, collections::HashSet};

use game_structs::Vec3;
use clap::Parser;
use rocket::{routes, post, State, serde::json::Json};

static WORLD_SIZE: f32 = 1024.; // The size of the total world
static MAX_PLAYERS: usize = 100; // The max players we want on a server
static BORDER_BUFFER_SIZE: f32 = 0.1; // The size of the buffer between which a player will be on both servers as a percentage of total size
static SERVER_ADDRESSES: [&str; 2] = ["http://127.0.0.1:8000", "http://127.0.0.1:8001"];

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let args = Args::parse();

    let session = Arc::new(RwLock::new(Server::Num(0, 0))); // Start at one server for entire world
    let session1 = session.clone();

    // Launch restructuring thread
    let restructuring_handle = thread::spawn(move || {
        restructure_servers(session1);
    });

    let figment = rocket::Config::figment()
        .merge(("port", args.port));

    rocket::custom(figment)
        .mount("/", routes![get_server])
        .manage(session)
        .launch().await?;

    restructuring_handle.join().expect("Failed to join restructuring thread.");

    Ok(())
}

/// Every 10 seconds redistribute servers based on current player count
pub fn restructure_servers(session: Session) {
    // A list of free servers
    let mut free_servers = vec![false, true]; // First server starts out as used, every other one is free
    loop {
        {
            // Get population numbers from servers
            let mut server = session.write().unwrap();
            
            // Update server populations
            server.update_population();

            // Run restructuring to free up servers
            server.restructure_free(&mut free_servers);

            // Run restructuring to allocate servers if nessacary
            server.restructure_allocate(&mut free_servers);
        }

        // Sleep for 10 seconds
        thread::sleep(std::time::Duration::from_secs(10));
    }
}

#[post("/get_server", format = "json", data = "<position>")]
fn get_server(position: Json<Vec3>, session: &State<Session>) -> String {
    let server_index = session.read().unwrap().query(*position + (WORLD_SIZE / 2.), WORLD_SIZE); // Add by WORLD_SIZE / 2 to put everything in positive coord system
    serde_json::to_string(&server_index).unwrap()
}


#[derive(Parser, Debug)]
#[clap(name = "Server")]
struct Args {
    /// The port to run on
    #[clap(short, long)]
    port: i32,
}

pub type Session = Arc<RwLock<Server>>;

#[derive(Debug)]
pub enum Server {
    Octree([[[Box<Server>; 2]; 2]; 2]),
    Num(usize, usize) // Contains index and population
}

impl Server {
    /// Get smallest server this position is inside
    pub fn query(&self, position: Vec3, size: f32) -> HashSet<usize> {
        match self {
            Self::Octree(a) => {
                let half_size = size / 2.;
                let (x_index, y_index, z_index) = (((position.x / half_size) as usize).clamp(0, 1), ((position.y / half_size) as usize).clamp(0, 1), ((position.z / half_size) as usize).clamp(0, 1));
                let mut main = a[x_index][y_index][z_index].query(position, half_size);
                if ((position.x - half_size) / half_size).abs() < BORDER_BUFFER_SIZE {
                    // X crossover, get other chunk
                    let x_adj_index = if (position.x % half_size) as usize == 0 {1} else {0};
                    main.extend(&a[x_adj_index][y_index][z_index].query(position, half_size));
                }
                if ((position.y - half_size) / half_size).abs() < BORDER_BUFFER_SIZE {
                    // Y crossover, get other chunk
                    let y_adj_index = if (position.y % half_size) as usize == 0 {1} else {0};
                    main.extend(&a[x_index][y_adj_index][z_index].query(position, half_size));
                }
                if ((position.z - half_size) / half_size).abs() < BORDER_BUFFER_SIZE {
                    // Z crossover, get other chunk
                    let z_adj_index = if (position.z % half_size) as usize == 0 {1} else {0};
                    main.extend(&a[x_index][y_index][z_adj_index].query(position, half_size));
                }
                main
            },
            Self::Num(i, _) => [*i].into_iter().collect()
        }
    }

    /// Go through each server and get an updated population count
    pub fn update_population(&mut self) {
        match self {
            Self::Octree(a) => {
                for x in a {
                    for y in x {
                        for z in y {
                            z.update_population();
                        }
                    }
                }
            },
            Self::Num(i, pop) => {
                // Update population of this server
                *pop = reqwest::blocking::get(format!("{}/get_num_players", crate::SERVER_ADDRESSES[*i]))
                    .unwrap().json().unwrap();
            }
        }
    }

    // Try to free up servers based on population numbers based on population numbers
    pub fn restructure_free(&mut self, free_servers: &mut Vec<bool>) {
        // Attempt to merge two blocks (pop is population of block1, index is index of block1) SUPER UGLY
        fn try_merge(parent_block: &mut [[[Box<Server>; 2]; 2]; 2], block1_coords: [usize; 3], block2_coords: [usize; 3], free_servers: &mut Vec<bool>) {
            if let Some(index) = parent_block[block1_coords[0]][block1_coords[1]][block1_coords[2]].get_index() {
                let mut pop = parent_block[block1_coords[0]][block1_coords[1]][block1_coords[2]].get_population().unwrap();
                if !parent_block[block2_coords[0]][block2_coords[1]][block2_coords[2]].is_octree() && parent_block[block2_coords[0]][block2_coords[1]][block2_coords[2]].get_index().unwrap() != index { // If adjacent block is not an octree and is not already merged with this block
                    if parent_block[block2_coords[0]][block2_coords[1]][block2_coords[2]].get_population().unwrap() + pop < MAX_PLAYERS { // Merge into this block and free server
                        pop += parent_block[block2_coords[0]][block2_coords[1]][block2_coords[2]].get_population().unwrap();
                        free_servers[parent_block[block2_coords[0]][block2_coords[1]][block2_coords[2]].get_index().unwrap()] = true; // Free the server
                        parent_block[block1_coords[0]][block1_coords[1]][block1_coords[2]].try_update(index, pop);
                        parent_block[block2_coords[0]][block2_coords[1]][block2_coords[2]].try_update(index, pop);
                    }
                }
            }
        }

        match self {
            Self::Octree(a) => {
                // Loop through octree to see if we can combine blocks
                let (mut one_block, first_index) = (true, a[0][0][0].get_index().unwrap());
                for x in 0..2 {
                    for y in 0..2 {
                        for z in 0..2 {
                            if a[x][y][z].is_octree() {
                                a[x][y][z].restructure_free(free_servers);
                            } else {
                                // There are 3 adjacent blocks for each block
                                // Other block along x
                                let adj_x = if x == 0 {1} else {0};
                                try_merge(a, [x, y, z], [adj_x, y, z], free_servers);
                                
                                // Other block along y
                                let adj_y = if y == 0 {1} else {0};
                                try_merge(a, [x, y, z], [x, adj_y, z], free_servers);

                                // Other block along z
                                let adj_z = if z == 0 {1} else {0};
                                try_merge(a, [x, y, z], [x, y, adj_z], free_servers);
                            }

                            if a[x][y][z].is_octree() || a[x][y][z].get_index().unwrap() != first_index {
                                one_block = false;
                            }
                        }
                    }
                }

                if one_block {
                    // Merge into one block
                    *self = Self::Num(first_index, a[0][0][0].get_population().unwrap());
                }
            },
            Self::Num(_, _) => {} // If we are just one server, nothing we can do
        }
    }

    // Allocate more servers if nessacary and more are availiable
    pub fn restructure_allocate(&mut self, free_servers: &mut Vec<bool>) {
        /// Try to split a block with the free servers availiable (naievely use all availiable free servers we need)
        pub fn try_split(block: &mut Server, free_servers: &mut Vec<bool>) {
            if block.is_octree() {return;}
            let index = block.get_population().unwrap();
            let mut a = [[[Server::Num(0, 0), Server::Num(0, 0)], [Server::Num(0, 0), Server::Num(0, 0)]], [[Server::Num(0, 0), Server::Num(0, 0)], [Server::Num(0, 0), Server::Num(0, 0)]]];
            // Assign servers and populations
            #[allow(clippy::needless_range_loop)]
            for x in 0..2 {
                for y in 0..2 {
                    for z in 0..2 {
                        if x == 0 && y == 0 && z == 0 { // Keep index for first server
                            a[x][y][z].try_update(index, 0);
                        } else {
                            let free_server = free_servers.iter().enumerate().find(|(_, b)| **b);
                            if let Some((index, _)) = free_server {
                                a[x][y][z].try_update(index, 0);
                                free_servers[index] = false;
                            } else {
                                a[x][y][z].try_update(index, 0);
                            }
                        }
                    }
                }
            }
        }

        match self {
            Self::Octree(a) => {
                // Loop through octree to see if we can combine blocks
                for x in a{
                    for y in x {
                        for z in y {
                            if z.is_octree() {
                                z.restructure_allocate(free_servers);
                            } else if z.get_population().unwrap() > MAX_PLAYERS {
                                // Try to split
                                try_split(z, free_servers);
                            }
                        }
                    }
                }
            },
            Self::Num(_, pop) => {
                if *pop > MAX_PLAYERS {
                    // Try to split
                    try_split(self, free_servers);
                }
            }
        }
    }

    pub fn is_octree(&self) -> bool {
        match self {
            Self::Octree(_) => true,
            Self::Num(_, _) => false
        }
    }

    pub fn get_population(&self) -> Option<usize> {
        match self {
            Self::Octree(_) => None,
            Self::Num(_, pop) => Some(*pop)
        }
    }

    pub fn get_index(&self) -> Option<usize> {
        match self {
            Self::Octree(_) => None,
            Self::Num(i, _) => Some(*i)
        }
    }

    pub fn try_update(&mut self, new_index: usize, new_pop: usize) {
        match self {
            Self::Octree(_) => {},
            Self::Num(i, pop) => {
                *i = new_index;
                *pop = new_pop;
            }
        }
    }
}