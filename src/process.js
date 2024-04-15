const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;

window.addEventListener("DOMContentLoaded", () => {
	let queryString = window.location.search; //get raw url params
	console.log(queryString);
	let urlParams = new URLSearchParams(queryString); //parse url params
    let id = urlParams.get('id'); //get id param
	console.log(id);
    if(id == null){ //check if id exists
        //show error
    } else {
        let stages = get_enroll_stages(); //how many enroll stages will the scanner perform
        //print "Please press your finger (stages) times making sure it blinks each time."
        //return if success
    }
});

async function get_enroll_stages(){
    return await invoke('get_device_enroll_stages');
}