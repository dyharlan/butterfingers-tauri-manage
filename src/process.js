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
        document.getElementById("proc-num").innerHTML = "no id";
    } else {
        get_enroll_stages(id); //how many enroll stages will the scanner perform
        //print "Please press your finger (stages) times making sure it blinks each time."
        //return if success
        //invoke('');
    }
});

async function get_enroll_stages(id) {
    let stages = await invoke('get_device_enroll_stages');
    document.getElementById("proc-num").innerHTML = "Please press your finger " + stages + " times making sure it blinks each time.";
    let result = await invoke('enroll_proc', {emp:id});
}