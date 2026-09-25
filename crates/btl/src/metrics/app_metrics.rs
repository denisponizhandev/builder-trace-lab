use std::sync::Arc;

use prometheus::{
    Encoder, Histogram, HistogramOpts, IntCounter, IntGauge, Opts, Registry, TextEncoder,
};

#[derive(Clone)]
pub struct AppMetrics {
    inner: Arc<AppMetricsInner>,
}

struct AppMetricsInner {
    registry: Registry,
    bundles_received_total: IntCounter,
    bundles_accepted_total: IntCounter,
    bundles_rejected_total: IntCounter,
    bundles_simulation_started_total: IntCounter,
    bundles_processed_total: IntCounter,
    bundles_failed_total: IntCounter,
    simulation_queue_depth: IntGauge,
    simulation_queue_capacity: IntGauge,
    simulation_in_flight: IntGauge,
    result_queue_depth: IntGauge,
    queue_wait_seconds: Histogram,
    simulation_duration_seconds: Histogram,
    bundle_end_to_end_seconds: Histogram,
    http_response_duration_seconds: Histogram,
    db_write_duration_seconds: Histogram,
}

impl AppMetrics {
    pub fn new(mode: &str) -> Self {
        let registry = Registry::new();

        let bundles_received_total =
            register_counter(&registry, "bundles_received_total", "valid bundle jobs before admission", mode);
        let bundles_accepted_total =
            register_counter(&registry, "bundles_accepted_total", "jobs accepted into simulation queue", mode);
        let bundles_rejected_total =
            register_counter(&registry, "bundles_rejected_total", "jobs rejected (queue full)", mode);
        let bundles_simulation_started_total = register_counter(
            &registry,
            "bundles_simulation_started_total",
            "simulation worker started processing a job",
            mode,
        );
        let bundles_processed_total =
            register_counter(&registry, "bundles_processed_total", "successful bundle result storage", mode);
        let bundles_failed_total =
            register_counter(&registry, "bundles_failed_total", "infrastructure storage failure", mode);

        let simulation_queue_depth =
            register_gauge(&registry, "simulation_queue_depth", "simulation queue depth", mode);
        let simulation_queue_capacity =
            register_gauge(&registry, "simulation_queue_capacity", "simulation queue capacity", mode);
        let simulation_in_flight =
            register_gauge(&registry, "simulation_in_flight", "simulations currently running", mode);
        let result_queue_depth =
            register_gauge(&registry, "result_queue_depth", "result channel depth", mode);

        let queue_wait_seconds = register_histogram(
            &registry,
            "queue_wait_seconds",
            "time from job receive to simulation start",
            mode,
        );
        let simulation_duration_seconds = register_histogram(
            &registry,
            "simulation_duration_seconds",
            "mock simulation work duration",
            mode,
        );
        let bundle_end_to_end_seconds = register_histogram(
            &registry,
            "bundle_end_to_end_seconds",
            "received_at to successful storage",
            mode,
        );
        let http_response_duration_seconds = register_histogram(
            &registry,
            "http_response_duration_seconds",
            "eth_sendBundle handler duration",
            mode,
        );
        let db_write_duration_seconds = register_histogram(
            &registry,
            "db_write_duration_seconds",
            "single bundle INSERT duration",
            mode,
        );

        Self {
            inner: Arc::new(AppMetricsInner {
                registry,
                bundles_received_total,
                bundles_accepted_total,
                bundles_rejected_total,
                bundles_simulation_started_total,
                bundles_processed_total,
                bundles_failed_total,
                simulation_queue_depth,
                simulation_queue_capacity,
                simulation_in_flight,
                result_queue_depth,
                queue_wait_seconds,
                simulation_duration_seconds,
                bundle_end_to_end_seconds,
                http_response_duration_seconds,
                db_write_duration_seconds,
            }),
        }
    }

    pub fn record_received(&self) {
        self.inner.bundles_received_total.inc();
    }

    pub fn record_accepted(&self) {
        self.inner.bundles_accepted_total.inc();
    }

    pub fn record_rejected(&self) {
        self.inner.bundles_rejected_total.inc();
    }

    pub fn record_simulation_started(&self) {
        self.inner.bundles_simulation_started_total.inc();
    }

    pub fn record_processed(&self) {
        self.inner.bundles_processed_total.inc();
    }

    pub fn record_failed(&self) {
        self.inner.bundles_failed_total.inc();
    }

    pub fn observe_queue_wait_seconds(&self, seconds: f64) {
        self.inner.queue_wait_seconds.observe(seconds);
    }

    pub fn observe_simulation_duration_seconds(&self, seconds: f64) {
        self.inner.simulation_duration_seconds.observe(seconds);
    }

    pub fn observe_bundle_end_to_end_seconds(&self, seconds: f64) {
        self.inner.bundle_end_to_end_seconds.observe(seconds);
    }

    pub fn observe_http_response_duration_seconds(&self, seconds: f64) {
        self.inner.http_response_duration_seconds.observe(seconds);
    }

    pub fn observe_db_write_duration_seconds(&self, seconds: f64) {
        self.inner.db_write_duration_seconds.observe(seconds);
    }

    pub fn set_simulation_queue_depth(&self, depth: i64) {
        self.inner.simulation_queue_depth.set(depth);
    }

    pub fn set_simulation_queue_capacity(&self, capacity: i64) {
        self.inner.simulation_queue_capacity.set(capacity);
    }

    pub fn set_simulation_in_flight(&self, count: i64) {
        self.inner.simulation_in_flight.set(count);
    }

    pub fn inc_simulation_in_flight(&self) {
        self.inner.simulation_in_flight.inc();
    }

    pub fn dec_simulation_in_flight(&self) {
        self.inner.simulation_in_flight.dec();
    }

    pub fn set_result_queue_depth(&self, depth: i64) {
        self.inner.result_queue_depth.set(depth);
    }

    pub fn encode(&self) -> String {
        let metric_families = self.inner.registry.gather();
        let mut buffer = Vec::new();
        TextEncoder::new()
            .encode(&metric_families, &mut buffer)
            .expect("prometheus text encode");
        String::from_utf8(buffer).expect("prometheus text utf8")
    }
}

fn register_counter(registry: &Registry, name: &str, help: &str, mode: &str) -> IntCounter {
    let counter = IntCounter::with_opts(bundle_opts(name, help, mode)).expect("counter opts");
    registry
        .register(Box::new(counter.clone()))
        .expect("register counter");
    counter
}

fn register_gauge(registry: &Registry, name: &str, help: &str, mode: &str) -> IntGauge {
    let gauge = IntGauge::with_opts(bundle_opts(name, help, mode)).expect("gauge opts");
    registry
        .register(Box::new(gauge.clone()))
        .expect("register gauge");
    gauge
}

fn register_histogram(registry: &Registry, name: &str, help: &str, mode: &str) -> Histogram {
    let histogram = Histogram::with_opts(
        HistogramOpts::new(name, help)
            .const_label("source", "bundle")
            .const_label("mode", mode),
    )
    .expect("histogram opts");
    registry
        .register(Box::new(histogram.clone()))
        .expect("register histogram");
    histogram
}

fn bundle_opts(name: &str, help: &str, mode: &str) -> Opts {
    Opts::new(name, help)
        .const_label("source", "bundle")
        .const_label("mode", mode)
}