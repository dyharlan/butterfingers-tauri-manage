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
use futures::io::Window;
use tauri::Window as TauriWindow;


use libfprint_rs::{
  FpPrint, 
  FpDevice,
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
   return unenrolled;        
}

#[tauri::command]
fn count(window: tauri::Window) {
  thread::spawn(move || {
    let mut times = 0;
    let ten_millis = time::Duration::from_millis(1000);
    while times <= 5 {
      window.emit("num", times.to_string());
      times+=1;
      thread::sleep(ten_millis);
    }
  });
  
}
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet,enumerate_unenrolled_employees,count])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
