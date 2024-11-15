use std::{path::PathBuf, ffi::OsStr, fs, sync::Arc};
use serde_json::{Value, json, Map};
use tokio::sync::Mutex;
use warp::{Filter, hyper::StatusCode, Reply, Rejection};

type CACHE = Arc<Mutex<Vec<Value>>>;

pub fn routes (cache: CACHE) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {

	let cors = warp::cors()
				.allow_any_origin();
				
	let base_filter = warp::path("currency");

	let ip2_filter = warp::path("ip2-services");

	let rows_filter = warp::any().map(move || cache.clone());

	let get_meta_data_filter = base_filter
	.and(warp::get())
	.and(warp::path("meta"))
	.and(rows_filter.clone())
	.and_then(handle_meta)
	.with(cors.clone());

	let get_filter = base_filter
	.and(warp::get())
	.and(warp::query::<Map<String, Value>>())
	.and(rows_filter.clone())
	.and_then(handle_get)
	.with(cors.clone());

	let post_filter = base_filter
	.and(warp::post())
	.and(warp::body::json())
	.and_then(handle_post);

	let ip2_currency_get_filter = ip2_filter
	.and(warp::path("ip2currency.aspx"))
	.and(warp::get())
	.and(warp::query::<Map<String, Value>>())
	.and(rows_filter.clone())
	.and_then(handle_get);

	let ip2_dmd_get_filter = ip2_filter
	.and(warp::path("ip2dmdproxy.aspx"))
	.and(warp::get())
	.and(warp::query::<Map<String, Value>>())
	.and_then(handle_dmd);

	let files = warp::fs::dir("./sources")
				.with(warp::compression::gzip());

	ip2_dmd_get_filter.or(ip2_currency_get_filter).or(get_meta_data_filter).or(get_filter).or(post_filter).or(files)
	
}

async fn handle_get(payload: Map<String, Value>, rows: CACHE) -> Result<impl Reply, Rejection> {

	let value: Value;
	let mut status_code = StatusCode::OK;

	if let Some(action) = payload.get("action") {

		if action == "getRows" {

			let rows = get_rows(rows).await;
	
			value = serde_json::to_value(rows).unwrap();
		}
		else if action == "getRow" {

			let key_value = payload.get("key");

			if let Some(key) = key_value {

				let k_value: Value = serde_json::from_str(key.as_str().unwrap()).unwrap();
				
				let id = k_value.get("code").unwrap().as_str().unwrap();
				value = get_row(rows, id).await;

				if value.get("code").is_none() {
					status_code = StatusCode::NOT_ACCEPTABLE;
				}
				
			}
			else {
				value = json!({
					"message": "invalid key"
				});

				status_code = StatusCode::NOT_ACCEPTABLE;
			}

		}
		else if action == "getMeta" {
			
			let meta_data = get_meta_data(rows).await;

			value = serde_json::to_value(meta_data).unwrap();
		}
		else {
			value = json!({"message": "action not found"});
			status_code = StatusCode::NOT_FOUND;
		}
	}
	else {
		value = json!({"message": "action missing"});
		status_code = StatusCode::NOT_FOUND;
	}

	let json_reply = warp::reply::json(&value);

	let response =  warp::reply::with_status(json_reply, status_code);

	Ok(response)
}

async fn handle_post(body: Value) -> Result<impl Reply, Rejection> {

	let response = warp::reply::json(&json!({}));

	Ok(response)
}

async fn handle_meta(rows: CACHE) -> Result<impl Reply, Rejection> {

	let result = get_meta_data(rows).await;

	let response = warp::reply::json(&result);

	Ok(response)
}

async fn handle_dmd(payload: Map<String, Value>) -> Result<impl Reply, Rejection> {

	let result: Value;

	let dmd_value = payload.get("dmd");

	if let Some(dmd) = dmd_value {

		let r = shellexpand::full(&"$POPATH/../projects/portfolio_6").unwrap();

		let os_str = OsStr::new(r.as_ref());

		let mut file_path = PathBuf::from(os_str);

		let dmd_str = dmd.as_str().unwrap();

		file_path.push(dmd_str);

		if file_path.exists() {
			let data = fs::read_to_string(file_path).unwrap();

			result = serde_json::from_str(&data).unwrap();
		}
		else {
			result = json!({
				"message": "Invalid path"
			})
		}
		
	}
	else {
		result = json!({
			"message": "File not found"
		});
	}

	let response = warp::reply::json(&result);

	Ok(response)
}

async fn handle_file(payload: Map<String, Value>) -> Result<impl Reply, Rejection> {

	let result: Value;

	let dmd_value = payload.get("dmd");

	if let Some(dmd) = dmd_value {

		let r = shellexpand::full(&"$POPATH/../projects/portfolio_6").unwrap();

		let os_str = OsStr::new(r.as_ref());

		let mut file_path = PathBuf::from(os_str);

		let dmd_str = dmd.as_str().unwrap();

		file_path.push(dmd_str);

		if file_path.exists() {
			let data = fs::read_to_string(file_path).unwrap();

			result = serde_json::from_str(&data).unwrap();
		}
		else {
			result = json!({
				"message": "Invalid path"
			})
		}
		
	}
	else {
		result = json!({
			"message": "File not found"
		});
	}

	let response = warp::reply::json(&result);

	Ok(response)
}

async fn get_rows(rows: CACHE) -> Value {

	let mut cache = rows.lock().await;

	if cache.len() == 0 {

		let req = reqwest::get("http://www.floatrates.com/daily/dkk.json");

		let result: Map<String, Value> = req.unwrap().json().unwrap();

		for (_, value) in result.iter() {

			let clone = value.clone();
			cache.push(clone);
		}
	}
	
	json!({
		"rows": cache.clone(),
		"totalRows": cache.len()
	})
	
}

async fn get_row(rows: CACHE, id: &str) -> Value {

	let object_rows = get_rows(rows).await;

	let mut result: Value = json!({"message": "Currency not found"});

	if let Some(rows) = object_rows.get("rows") {

		let rows = rows.as_array().unwrap();

		let row_value = rows.iter().find(|search| {

			let search_code = search.get("code").unwrap().as_str().unwrap();
			search_code == id
		});

		if let Some(row) = row_value {

			result = json!(row);
		}
	}
	
	result
}	

async fn get_meta_data(rows: CACHE) -> Value {

	let object_rows = get_rows(rows).await;

	let mut result: Value = json!({});

	if let Some(rows) = object_rows.get("rows") {

		let rows = rows.as_array().unwrap();

		let row_value = rows.first();

		if let Some(row) = row_value {

			let map: Map<String, Value>  = serde_json::from_value(row.clone()).unwrap();

			let mut columns = vec![];

			for (key, _) in map.iter() {

				columns.push(json!({
					"name": key.clone()
				}))
			}

			result = json!({
				"model": {
					"entities": [
						{
							"name": "currency",
							"title": "Currency",
							"idColumn": "code",
							"primaryKey": [],
							"columns": columns
						}
					]
				}
			});
		}
	}

	return result;

}
