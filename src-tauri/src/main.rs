// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::{
  env, 
  fs::OpenOptions,
  io::{self, Write}, 
  sync::{Arc, Mutex},
  thread,
  time,
};
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use futures::io::Window;
use tauri::State;


use libfprint_rs::{
  FpPrint, 
  FpDevice,
  FpContext,
};
    
use sqlx::{
  MySqlPool,
  Row,
};
use uuid::Uuid;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
#[derive(Serialize, Deserialize)]
struct Employee {
  emp_id: u64,
  fname: String,
  lname: String,
}
#[tauri::command]
async fn enumerate_unenrolled_employees() -> String {
  dotenvy::dotenv().unwrap();
  let pool = MySqlPool::connect(&env::var("DATABASE_URL").unwrap()).await.unwrap(); 
  let result = sqlx::query!("CALL enumerate_unenrolled_employees_json")
  .fetch_all(&pool)
  .await.unwrap();
  
   if result.is_empty() {
      println!("No unenrolled employees found");
      return String::from("{\"error\":\"No unenrolled employees found\"}");
   }
   let mut unenrolled: String = String::from("");
   for (_,row) in result.iter().enumerate() {
   	let json = row.get::<serde_json::Value, usize>(0);
   	unenrolled = json.to_string();
   }
   pool.close().await;
   return unenrolled;        
}

#[tauri::command]
fn enroll_proc(emp: String,device: State<Note>) -> String {
  let emp_num = match emp.trim().parse::<u64>() {
    Ok(num) => num,
    Err(_) => return json!({
        "responsecode" : "failure",
        "body" : "Invalid employee ID",
    }).to_string(),
  };
  let pool = futures::executor::block_on(async {
    MySqlPool::connect(&env::var("DATABASE_URL").unwrap()).await;
  });
  /*
  * Get emp_id and check if it already is enrolled.
  */
  let fp_scanner = device.0.lock().unwrap();
  //open the fingerprint scanner
  fp_scanner.open_sync(None).expect("Device could not be opened");
    
  //create a template for the user
  let template = FpPrint::new(&fp_scanner);
  //set the username of the template to the uuid generated
  //generates a random uuid
  let uuid = Uuid::new_v4();
  template.set_username(&uuid.to_string()); 
  println!("Username of the fingerprint: {}", template.username().expect("Username should be included here"));
  let counter = Arc::new(Mutex::new(0));
  let new_fprint = fp_scanner
  .enroll_sync(template, None, Some(enroll_cb), None)
  .expect("Fingerprint could not be enrolled");
  //close the fingerprint scanner
  fp_scanner.close_sync(None).expect("Device could not be closed");
  println!("Total enroll stages: {}", counter.lock().unwrap());
  //create a file to store the fingerprint in (at the root folder, which is securely located in the home directory)
  let mut file = OpenOptions::new()
  .write(true)
  .create(true)
  .open(dirs::home_dir()
    .expect("Failed to get home directory")
    .join(format!("print/fprint_{}",uuid)))
    .expect("Creation of file failed");

  //fingerprint serialized for storage at the file location
  file.write_all(&new_fprint.serialize().expect("Could not serialize fingerprint"))
  .expect("Error: Could not store fingerprint to the txt file");
  let insert =futures::executor::block_on(async {
    sqlx::query!("CALL save_fprint_identifier(?,?)",emp_num,uuid.to_string())
      .execute(&pool) //execute the stored prodcedure
      .await; //wait for the query to finish
    match insert.rows_affected() { //check how many rows were affected by the stored procedure that was previously queried
      0 => println!("No rows affected"),
      _ => println!("Rows affected: {}", insert.rows_affected()),
    }
    pool.close().await; //close the connection to the database
  });
  
        
  Ok(()) //return the function with no errors
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

fn main() {
    tauri::Builder::default()
        .manage(Note(Mutex::new(FpContext::new().devices().remove(0))))
        .invoke_handler(tauri::generate_handler![greet,enumerate_unenrolled_employees,count,get_device_enroll_stages])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
