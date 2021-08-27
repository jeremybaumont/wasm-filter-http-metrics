use log::debug;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use std::time::SystemTime;
use std::str;

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(HttpMetricsRoot) });
}

struct HttpMetricsRoot;

impl Context for HttpMetricsRoot {}

impl RootContext for HttpMetricsRoot {
    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn create_http_context(&self, context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(HTTPMetrics::new(context_id)))
    }
}


#[derive(Debug)]
struct HTTPMetrics {
    context_id: u32,
    time: SystemTime,
    latency: u64,
    rqDurationMetricId: u32,
}

impl Context for HTTPMetrics {}

impl HTTPMetrics {
    fn new(context_id: u32) -> Self {
        return Self {
            context_id,
            time: SystemTime::UNIX_EPOCH,
            latency: 0,
            rqDurationMetricId: proxy_wasm::hostcalls::define_metric(MetricType::Histogram, "rqDuration").unwrap(),
        }
    }
}

impl HttpContext for HTTPMetrics {
    fn on_http_response_body(&mut self, _body_size: usize, _end_of_stream: bool) -> Action {

      if let Some(metrics_tag_utf8) = self.get_property(vec![
                "route_metadata",
            ]) {
         let metrics_tag = match str::from_utf8(&metrics_tag_utf8) {
            Ok(ak) => ak,
            Err(e) => panic!("Error: {}", e),
        };

        debug!("metrics tag from route metadata: {}", metrics_tag.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-'));
        if _end_of_stream {
            if let Ok(curr_time) = proxy_wasm::hostcalls::get_current_time() {
                if let Ok(dur) = curr_time.duration_since(self.time) {
                    let nanos = dur.subsec_nanos() as u64;
                    let ms = (1000*1000*1000 * dur.as_secs() + nanos)/(1000 * 1000);
                    self.latency = ms; 
                }
            }
            debug!("{:?}", self);

            proxy_wasm::hostcalls::record_metric(self.rqDurationMetricId, self.latency).unwrap();
        }
     } else {
        debug!("could not find route metadata");
     }
       
     Action::Continue
    }
}
