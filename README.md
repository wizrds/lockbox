# Lockbox

Lockbox is a secure API key management system designed to allow API key authentication for Rest APIs.

## Quick Start

To run Lockbox using Docker, execute the following command:

```bash
docker run -p 8087:8087 ghcr.io/wizrds/lockbox:0.1.0 serve --migrations
```

## Installation with Helm

To install Lockbox using Helm, you can use the following command:

```bash
helm upgrade --install lockbox oci://ghcr.io/wizrds/lockbox/charts/lockbox --version 0.1.0
```

For the available values please refer to the [values.yaml](deploy/helm/lockbox/values.yaml) file.

For API reference, visit the [API Documentation](http://localhost:8087/.well-known/docs) once the container is running.

## License
This project is licensed under ISC License.

## Support & Feedback
If you encounter any issues or have feedback, please open an issue.

Made with ❤️ by Tim Pogue
