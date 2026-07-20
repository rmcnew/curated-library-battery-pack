// JavaScript version of web_service API

// generate a UUID v4
function uuid_v4() {
  let d = new Date().getTime(); // timestamp
  let d2 = ((typeof performance !== 'undefined') && performance.now && (performance.now()*1000)) || 0; // time in microseconds since page-load or 0 if unsupported
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
    let r = Math.random() * 16; // random number between 0 and 16
    if (d > 0) { // use timestamp until depleted
      r = (d + r) % 16 | 0;
      d = Math.floor(d / 16);
    } else { // use microseconds since page-load if supported
      r = (d2 + r) % 16 | 0;
      d2 = Math.floor(d2 / 16);
    }
    return (c === 'x' ? r : (r & 0x3 | 0x8)).toString(16);
  })
}

// get the system timestamp in ISO 8601 format
function timestamp_now() {
  let date = new Date();
  return date.toISOString();
}

// is an argument defined?
export function defined(arg) {
  return !(arg === null || arg === undefined || arg === '');
}

// Constants used in JSON objects
const request_id = "request_id";
const timestamp = "timestamp";

// get web_service server URL
export function server_url() {
  return document.location.origin;
}

// PUT method
async function put_method(url, request) {
  console.log(`PUTting request to URL: ${url}`);
  console.log(request);
  let params = {
    method: 'PUT',
    headers: {
      'Content-type': 'application/json; charset=UTF-8'
    },
    body: JSON.stringify(request)
  };  
  const response = await fetch(url, params);
  if (!response.ok) {
    const error_text = await response.text();
    let error_message = `Error: status: ${response.status}, message: ${error_text}`;
    console.error(error_message);
    window.alert(error_message);
    throw new Error(error_message);
  }
  const response_json = await response.json();
  console.log("put_method response_json is:");
  console.log(response_json);
  return response_json;  
}

// Status
const STATUS_PATH = "/api/v1/status";
const STATUS_API_URL = server_url() + STATUS_PATH;
/* Example StatusRequest JSON
{ 
  "request_id": "45fef50f-f6c6-4ac2-a9d4-ec33ba48b874",
  "timestamp": "2025-01-24T21:43:52.734157904Z"
}
*/
function status_request() {
  let request = {
    request_id: uuid_v4(), 
    timestamp: timestamp_now()};
  return request;
}

/* Example StatusResponse JSON
{
  "request":{
    "request_id": "45fef50f-f6c6-4ac2-a9d4-ec33ba48b874",
    "timestamp": "2025-01-24T21:43:52.734157904Z"
  },
  "timestamp": "2025-01-24T21:43:52.746901057Z"
}
*/

export async function status() {
  let request = status_request();
  let response = await put_method(STATUS_API_URL, request);
  return response;
}


