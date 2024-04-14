const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;
window.addEventListener("DOMContentLoaded", () => {
  //let queryString = window.location.search;
  //console.log(queryString);
  //let urlParams = new URLSearchParams(queryString);
  //console.log(urlParams.get('param'));
  //hello();
  enumerate_unenrolled_employees();
});
window.unlisten = await listen("num", (ev) => {
  document.querySelector("#tubol").textContent = ev.payload;
});
async function enumerate_unenrolled_employees(){
  //await invoke('count');
  let results = await invoke('enumerate_unenrolled_employees');
  let results_json = JSON.parse(results);
  for (var i = 0; i < results_json.length; i++){ //loop for each element
  	var emp = results_json[i];
  	if(emp.hasOwnProperty("error")){
  		console.log("error: "+emp['error']);
  		return;
  	}
  	if(emp.hasOwnProperty('emp_id')){
  		console.log("emp_id: "+ emp['emp_id']);
  	}
  	if(emp.hasOwnProperty('fname')){
  		console.log("first name : "+emp['fname']);
  	}
  	if(emp.hasOwnProperty('lname')){
  		console.log("last name: "+emp['lname']);
  	}			
  }
}
