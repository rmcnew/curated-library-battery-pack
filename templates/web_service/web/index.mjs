import * as web_service_client from '/web_service_client.mjs';

const status_request_button = document.getElementById("status-request-button");
const status_response_div = document.getElementById("status-response-div");

async function status_request() {
    let response = await web_service_client.status();
    if (Object.hasOwn(response, 'request')) {
        let request = response.request;
        if (Object.hasOwn(request, 'request_id') && Object.hasOwn(request, 'timestamp')) {
            status_response_div.textContent = "Server is responding:\n" + JSON.stringify(response);
        } else {
            status_response_div.textContent = "Malformed response from server (request missing elements):\n" + JSON.stringify(response);
        }
    } else {
        status_response_div.textContent = "Malformed response from server (no request found):\n" + JSON.stringify(response);
    }
}

status_request_button.addEventListener("click", status_request);

