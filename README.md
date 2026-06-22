# Lockbox

Lockbox is a secure API key management system designed to allow API key authentication for Rest APIs.

> **Note**: The versioning of this project is currently in early development, and breaking changes may occur inbetween minor versions until a stable 1.0.0 release is made. Please carefully review when upgrading to ensure compatibility with your existing setup.

## Quick Start

To run Lockbox using Docker, execute the following command:

```bash
docker run -p 8087:8087 ghcr.io/wizrds/lockbox:0.3.1 serve --migrations
```

## Installation with Helm

To install Lockbox using Helm, you can use the following command:

```bash
helm upgrade --install lockbox oci://ghcr.io/wizrds/lockbox/charts/lockbox --version 0.3.1
```

For the available values please refer to the [values.yaml](deploy/helm/lockbox/values.yaml) file.

For API reference, visit the [API Documentation](http://localhost:8087/.well-known/docs) once the container is running.

## License
This project is licensed under ISC License.

## Support & Feedback
If you encounter any issues or have feedback, please open an issue.

Made with ❤️ by Tim Pogue
