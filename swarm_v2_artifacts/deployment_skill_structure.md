# DeploymentSkill Modular Structure

## 📂 /core
- `IntentParser.ts`: Validates and normalizes input arguments.
- `StateEngine.ts`: Manages the "Desired vs Actual" state diffing.
- `DependencyResolver.ts`: Generates the Directed Acyclic Graph (DAG) for deployment.

## 📂 /providers
- `K8sProvider.ts`: Implementation for Kubernetes clusters.
- `AwsProvider.ts`: Implementation for AWS ECS/Lambda.
- `AzureProvider.ts`: Implementation for Azure Container Apps.

## 📂 /strategies
- `RollingUpdate.ts`: Logic for incremental replacement.
- `BlueGreen.ts`: Logic for environment switching.
- `Canary.ts`: Logic for traffic splitting and metric analysis.

## 📂 /telemetry
- `HealthMonitor.ts`: Interfaces with Prometheus/Datadog for verification.
- `AuditLogger.ts`: Records every state change for compliance.