import * as web_service_api from '/web_service_api.mjs';

// re-export the web_service_api.mjs exports to simplify the namespace usage for web_service web pages
export * from '/web_service_api.mjs';

// web_service web pages URLs
export const index_page = "index.html";


// GET method
async function get_method(url) {    
    let params = {
      method: 'GET',
      headers: {
        'Content-type': 'text/plain; charset=UTF-8'
      },      
    };  
    const response = await fetch(url, params);
    if (!response.ok) {
      const error_text = await response.text();
      let error_message = `Error: status: ${response.status}, message: ${error_text}`;
      log_error(error_message);
      throw new Error(error_message);
    }
    const response_text = await response.text();    
    return response_text;  
  }


// URL query string parameters
export function get_url_query_parameters() {
    const params = new URLSearchParams(window.location.search);
    return params;
}

