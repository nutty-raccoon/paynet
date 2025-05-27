# Observability Stack Documentation

This document describes the OpenTelemetry-based observability stack for the Paynet project.

## Configuration

The behaviour of each component of the stack is configured by static files at [./observability/](./observability).

## Architecture Overview

The observability stack provides comprehensive monitoring, logging, and tracing capabilities through a collection of industry-standard tools working together.

## Components

### OpenTelemetry Collector
**Role**: Central telemetry data collection and routing hub

The collector receives telemetry data (metrics, logs, traces) from all Paynet services via OTLP protocol and routes it to the appropriate backend systems. It handles data processing, batching, and export to multiple destinations.

### Prometheus
**Role**: Metrics storage and querying engine

Stores time-series metrics data and provides a query language (PromQL) for analyzing system performance and business metrics. Scrapes metrics from the OpenTelemetry Collector.

### Grafana
**Role**: Visualization and dashboarding platform

Provides web-based dashboards for visualizing metrics and logs. Connects to both Prometheus (for metrics) and Loki (for logs) to create comprehensive operational views.

### Jaeger
**Role**: Distributed tracing system

Tracks requests as they flow through multiple services, helping identify bottlenecks and understand system behavior. Receives trace data from the OpenTelemetry Collector.

### Loki
**Role**: Log aggregation and querying

Aggregates logs from all services and provides efficient querying capabilities. Designed to work seamlessly with Grafana for log visualization.

## Service Integration

All services are instrumented with OpenTelemetry through the shared open-telemetry-tracing crate: Node Service, Signer Service and Starknet Cashier.
Each service calls the library's initialization function during startup, which automatically configures tracing, metrics collection, and structured logging. The services then send telemetry data to the OpenTelemetry Collector via OTLP without requiring additional configuration.

Each service sends telemetry data to the OpenTelemetry Collector via OTLP.

## Deployment

The entire observability stack is deployed using Docker Compose and can be started alongside the main application. All components include health checks and proper service dependencies.

## Access Points

- **Grafana Dashboards**: http://localhost:3000 (admin/admin)
- **Prometheus UI**: http://localhost:9090
- **Jaeger Tracing**: http://localhost:16686

## Data Flow

```
Paynet Services → OpenTelemetry Collector → Prometheus/Jaeger/Loki → Grafana
```
