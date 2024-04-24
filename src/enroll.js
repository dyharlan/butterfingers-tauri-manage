const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;
const dialog = window.__TAURI__.dialog;
window.addEventListener("DOMContentLoaded", () => {
	//let queryString = window.location.search;
	//console.log(queryString);
	//let urlParams = new URLSearchParams(queryString);
	//console.log(urlParams.get('param'));
	//hello();
	document.getElementById("search").onkeyup = function () {
		searchEmp();
	};
	enumerate_unenrolled_employees();
});
async function enumerate_unenrolled_employees() {
	//await invoke('count');
	let results = await invoke('enumerate_unenrolled_employees');
	let results_json = JSON.parse(results);
	console.log(results_json);
	for (var i = 0; i < results_json.length; i++) { //loop for each element
		/*
		add logic that if the json array is empty, show that there are no unenrolled employees
		(depends if you want to leave the list blank or to show that there are no unenrolled employees)
		*/
		var emp = results_json[i];
		if (emp.hasOwnProperty("error")) {
			console.log("error: " + emp['error']);
			//hindi nag didisplay error sa page?
			return;
		}
		if (emp.hasOwnProperty('emp_id')) {
			console.log("emp_id: " + emp['emp_id']);
		}
		if (emp.hasOwnProperty('fname')) {
			console.log("first name : " + emp['fname']);
		}
		if (emp.hasOwnProperty('lname')) {
			console.log("last name: " + emp['lname']);
		}

		if (emp.hasOwnProperty('emp_id') && emp.hasOwnProperty('fname') && emp.hasOwnProperty('lname')) {
			addToList(emp['emp_id'], emp['fname'], emp['lname']);
		}
	}

}

function addToList(id, fname, lname) {
	var ul = document.getElementById("item-list");
	console.log(ul);
	var li = document.createElement("li");

	var p1 = document.createElement("p");
	var p2 = document.createElement("p");
	p1.appendChild(document.createTextNode("ID: " + id));
	p1.setAttribute("class", "emp-id")

	p2.appendChild(document.createTextNode("NAME: " + fname + " " + lname));
	p2.setAttribute("class", "emp-name")

	li.appendChild(p1);
	li.appendChild(p2);
	li.setAttribute("id", id);
	li.setAttribute("class", "item");
	li.onclick = function () {
		selectEmp(id, fname, lname);
	};

	ul.appendChild(li);
}

async function selectEmp(id, fname, lname) {
	const confirmed = await dialog.confirm("Are you sure about enrolling " + fname + " " + lname + "'s fingerprint?",
		{ title: "Confirm Selected Employee", okLabel: "Yes", });

	if (confirmed) {
		// window.location.href = "/enrollProcess.html?id=" + id;
		window.location.replace("/enrollProcess.html?id=" + id + "&empname=" + fname + "-" + lname);
	}
}

function searchEmp() {
	// Declare variables
	var input, filter, items, name, id, i, txtValueName, txtValueId;
	input = document.getElementById('search');
	filter = input.value.toUpperCase();
	items = document.getElementsByClassName('item');

	// Loop through all list items, and hide those who don't match the search query
	for (i = 0; i < items.length; i++) {
		name = items[i].getElementsByClassName("emp-name")[0]; // target emp-name
		id = items[i].getElementsByClassName("emp-id")[0]; // target emp-id
		txtValueName = name.textContent || name.innerText;
		txtValueId = id.textContent || id.innerText;
		if (txtValueName.toUpperCase().indexOf(filter) > -1 || txtValueId.toUpperCase().indexOf(filter) > -1) {
			items[i].style.display = "";
		} else {
			items[i].style.display = "none";
		}
	}
}