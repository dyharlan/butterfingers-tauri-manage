// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::{
    env,
    fs::OpenOptions,
    io::Write,
    sync::{Arc, Mutex},
};

use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use tauri::State;

use libfprint_rs::{FpContext, FpDevice, FpPrint};

use sqlx::{MySqlPool, Row};
use uuid::Uuid;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

#[derive(Serialize, Deserialize)]
struct Employee {
    emp_id: u64,
    fname: String,
    lname: String,
}
#[tauri::command]
async fn enumerate_unenrolled_employees() -> String {
    let database_url = match db_url() {
        Ok(url) => url,
        Err(e) => {
            return json!({
              "error": format!("DATABASE_URL not set: {}", e)
            })
            .to_string()
        }
    };

    println!("database_url: {}", database_url);

    let pool = match MySqlPool::connect(&database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            return json!({
              "error": format!("Could not connect to database: {}",e)
            })
            .to_string()
        }
    };

    let result = match sqlx::query!("CALL enumerate_unenrolled_employees_json")
        .fetch_all(&pool)
        .await
    {
        Ok(result) => result,
        Err(_) => {
            return json!({
              "error" : "Failed to execute query"
            })
            .to_string()
        }
    };

    pool.close().await;

    //println!("Found {} unenrolled employees", result.len());

    if result.is_empty() {
        println!("No unenrolled employees found");
        return json!({
          "error" : "No unenrolled employees found"
        })
        .to_string();
    }

    //println!("{:?}", result.get(0).unwrap());

    let mut unenrolled: String = String::from("");

    for row in result.iter() {
        let json = row.get::<serde_json::Value, usize>(0);
        unenrolled = json.to_string();
    }

    unenrolled
}

#[tauri::command]
fn enroll_proc(emp: String, device: State<Note>) -> String {
    let emp_num = match emp.trim().parse::<u64>() {
        Ok(num) => num,
        Err(_) => {
            return json!({
                "responsecode" : "failure",
                "body" : "Invalid employee ID",
            })
            .to_string()
        }
    };

    /*
     * Get emp_id and check if it already is enrolled.
     */

    // let result = match futures::executor::block_on(async {
    //   query_count(emp_num).await
    // }) {
    //   Ok(result) => result,
    //   Err(e) => return json!({
    //     "responsecode" : "failure",
    //     "body" : format!("Failed to execute query: {}",e),
    //   }).to_string()
    // };

    let fp_scanner = match device.0.lock() {
        Ok(fp_scanner) => fp_scanner,
        Err(_) => {
            return json!({
              "responsecode" : "failure",
              "body" : "Could not get device",
            })
            .to_string()
        }
    };

    //open the fingerprint scanner
    match fp_scanner.open_sync(None) {
        Ok(_) => (),
        Err(_) => {
            return json!({
              "responsecode" : "failure",
              "body" : "Could not open device",
            })
            .to_string()
        }
    }

    //create a template for the user
    let template = FpPrint::new(&fp_scanner);

    //generates a random uuid
    let uuid = Uuid::new_v4();

    //set the username of the template to the uuid generated
    template.set_username(&uuid.to_string());

    println!(
        "Username of the fingerprint: {}",
        template
            .username()
            .expect("Username should be included here")
    );

    let counter = Arc::new(Mutex::new(0));

    let new_fprint = match fp_scanner.enroll_sync(template, None, Some(enroll_cb), None) {
        Ok(new_fprint) => new_fprint,
        Err(_) => {
            fp_scanner
                .close_sync(None)
                .expect("Could not close fingerprint scanner");
            return json!({
              "responsecode" : "failure",
              "body" : "Could not enroll fingerprint",
            })
            .to_string();
        }
    };

    println!("Fingerprint has been scanned");

    //close the fingerprint scanner
    match fp_scanner.close_sync(None) {
        Ok(_) => (),
        Err(_) => {
            return json!({
              "responsecode" : "failure",
              "body" : "Could not close fingerprint scanner",
            })
            .to_string();
        }
    } //.expect("Device could not be closed");

    println!("Total enroll stages: {}", counter.lock().unwrap());

    let home_dir = match dirs::home_dir() {
        Some(home_dir) => home_dir,
        None => {
            return json!({
              "responsecode" : "failure",
              "body" : "Could not get home directory to store fingerprint",
            })
            .to_string()
        }
    };

    //create a file to store the fingerprint in (at the root folder, which is securely located in the home directory)
    let mut file = match OpenOptions::new()
        .write(true)
        .create(true)
        //.truncate(true) //suggested by clippy to add truncate because file is opened with create, but truncate behavior is not defined
        .open(home_dir.join(format!("print/fprint_{}", uuid)))
    {
        Ok(file) => file,
        Err(_) => {
            return json!({
              "responsecode" : "failure",
              "body" : "Could not create fingerprint file",
            })
            .to_string();
        }
    };
    //.expect("Creation of file failed");

    //serialize the fingerprint
    let new_fprint = match new_fprint.serialize() {
        Ok(new_fprint) => new_fprint.to_owned(),
        Err(_) => {
            return json!({
              "responsecode" : "failure",
              "body" : "Could not serialize fingerprint",
            })
            .to_string();
        }
    };

    //fingerprint serialized for storage at the file location
    match file.write_all(&new_fprint) {
        Ok(_) => (),
        Err(_) => {
            return json!({
              "responsecode" : "failure",
              "body" : "Could not write fingerprint to file",
            })
            .to_string();
        }
    }

    futures::executor::block_on(async {
        match save_fprint_identifier(&emp_num, &uuid.to_string()).await {
            Ok(_insert) => {
                println!("Fingerprint has been saved in the database");
                json!({
                  "responsecode" : "success",
                  "body" : "Successfully enrolled fingerprint",
                })
                .to_string()
            }
            Err(result) => json!({
              "responsecode" : "failure",
              "body" : result.to_string(),
            })
            .to_string(),
        }
    })
}

// async fn query_count(emp_id: u64) -> Result<(), String> {

//   let database_url = match db_url() {
//     Ok(url) => url,
//     Err(e) => return Err(format!("DATABASE_URL not set: {}", e)),
//   };

//   let pool = match MySqlPool::connect(&database_url).await {
//     Ok(pool) => pool,
//     Err(e) => return Err(e.to_string()),
//   };

//   let record = match sqlx::query!("SELECT COUNT(*) AS count_result FROM enrolled_fingerprints WHERE EMP_ID = ?", emp_id)
//     .fetch_one(&pool)
//     .await {
//       Ok(result) => {
//         if result.count_result == 1 {
//           pool.close().await;
//           return Err(json!({
//             "responsecode" : "failure",
//             "body" : "Employee already enrolled",
//           }).to_string());
//         }
//       },
//       Err(e) => return Err(e.to_string()),
//     };

//   pool.close().await; //close connection to database
//   Ok(())
// }

async fn save_fprint_identifier(emp_id: &u64, fprint_uuid: &str) -> Result<(), String> {
    let database_url = match db_url() {
        Ok(url) => url,
        Err(e) => return Err(format!("DATABASE_URL not set: {}", e)),
    };

    //connect to the database
    let pool = match MySqlPool::connect(&database_url).await {
        Ok(pool) => pool,
        Err(e) => return Err(e.to_string()),
    };

    //query the record_attendance_by_empid stored procedure (manual attendance)
    match sqlx::query!(
        "CALL save_fprint_identifier(?,?)",
        emp_id,
        fprint_uuid.to_string()
    )
    .execute(&pool)
    .await
    {
        Ok(row) => {
            pool.close().await;
            match row.rows_affected() {
                //check how many rows were affected by the stored procedure that was previously queried
                0 => println!("No rows affected"),
                _ => println!("Rows affected: {}", row.rows_affected()),
            }
        }
        Err(e) => return Err(e.to_string()),
    };
    //.expect("Could not retrieve latest attendance record");

    pool.close().await; //close connection to database
    Ok(()) //return from the function with no errors
}

fn db_url() -> Result<String, String> {
    match dotenvy::dotenv() {
        Ok(_) => (),
        Err(e) => return Err(format!("Failed to load .env file: {}", e)),
    }

    let db_type = match env::var("DB_TYPE") {
        Ok(db_type) => db_type,
        Err(_) => return Err("DB_TYPE not set".to_string()),
    };

    let db_username = match env::var("DB_USERNAME") {
        Ok(username) => username,
        Err(_) => return Err("DB_USERNAME not set".to_string()),
    };

    let db_password = match env::var("DB_PASSWORD") {
        Ok(password) => password,
        Err(_) => return Err("DB_PASSWORD not set".to_string()),
    };

    let hostname = match env::var("HOSTNAME") {
        Ok(name) => name,
        Err(_) => return Err("HOSTNAME not set".to_string()),
    };

    let db_port = match env::var("DB_PORT") {
        Ok(port) => port,
        Err(_) => return Err("DB_PORT not set".to_string()),
    };

    let db_name = match env::var("DB_NAME") {
        Ok(name) => name,
        Err(_) => return Err("DB_NAME not set".to_string()),
    };

    let db_params = match env::var("DB_PARAMS") {
        Ok(params) => params,
        Err(_) => return Err("DB_PARAMS not set".to_string()),
    };

    let database_url = format!(
        "{}://{}:{}@{}:{}/{}?{}",
        db_type, db_username, db_password, hostname, db_port, db_name, db_params
    );
    Ok(database_url)
}

pub fn enroll_cb(
    _device: &FpDevice,
    enroll_stage: i32,
    _print: Option<FpPrint>,
    _error: Option<libfprint_rs::GError>,
    _data: &Option<i32>,
) {
    //print enroll stage of the enroll function
    println!("Enroll_cb Enroll stage: {}", enroll_stage);
}

#[tauri::command]
fn get_device_enroll_stages(device: State<Note>) -> i32 {
    return device.0.lock().unwrap().nr_enroll_stage();
}

struct Note(Mutex<FpDevice>);

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .setup(|_app| Ok(()))
        .manage(Note(Mutex::new(FpContext::new().devices().remove(0))))
        .invoke_handler(tauri::generate_handler![
            enumerate_unenrolled_employees,
            enroll_proc,
            get_device_enroll_stages
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
