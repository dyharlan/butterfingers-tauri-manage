const { invoke } = window.__TAURI__.tauri;
import Database from "tauri-plugin-sql-api";

// mysql
const db = await Database.load("mysql://root:toorenia@192.168.100.23/pyfi_db");

const result = await db.execute("call enumerate_unenrolled_employees()");
window.addEventListener("DOMContentLoaded", () => {
  let queryString = window.location.search;
  console.log(queryString);
  let urlParams = new URLSearchParams(queryString);
  console.log(urlParams.get('param'));
  hello();
});

async function hello(){
  let hello = await invoke("hello");
  console.log(hello);
}
