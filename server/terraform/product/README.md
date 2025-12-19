# Terraform Modules for Hardware API

This folder contains the [Terraform] modules to deploy the
[Hardware API charm][hardware-api-charm].

The module uses the [Terraform Juju provider][juju-provider] to model the bundle
deployment onto any Kubernetes environment managed by [Juju].

## Module Structure

- [`main.tf`](main.tf): Defines the Juju application to be deployed.
- [`variables.tf`](variables.tf): Allows customization of the deployment.
- [`outputs.tf`](outputs.tf): Responsible for integrating the module with other
  Terraform modules, primarily by defining potential integration endpoints
  (charm integrations).
- [`versions.tf`](versions.tf): Defines the Terraform provider versions.

## Requirements

| Name                   | Version   |
| ---------------------- | --------- |
| [terraform][terraform] | >= 1.7.2  |
| [juju][juju-provider]  | ~> 0.17.0 |

## Providers

| Name                  | Version   |
| --------------------- | --------- |
| [juju][juju-provider] | ~> 0.17.0 |

## Modules

| Name         | Source               | Version |
| ------------ | -------------------- | ------- |
| hardware_api | [../charm](../charm) | n/a     |

## Resources

| Name                                                          | Type        |
| ------------------------------------------------------------- | ----------- |
| [juju_model.hardware_api][juju_model]                         | data source |
| [juju_application.nginx_ingress_integrator][juju_application] | resource    |
| [juju_integration.hardware_api_ingress][juju_integration]     | resource    |

## Inputs

| Name                     | Description                                                                       | Type   |
| ------------------------ | --------------------------------------------------------------------------------- | ------ |
| model                    | Reference to an existing model resource or data source for the model to deploy to | string |
| hardware_api             | n/a                                                                               | object |
| nginx_ingress_integrator | n/a                                                                               | object |

## Outputs

| Name                              | Description                                               |
| --------------------------------- | --------------------------------------------------------- |
| hardware_api_app_name             | Name of the deployed Hardware API application             |
| hardware_api_requires             |                                                           |
| hardware_api_provides             |                                                           |
| nginx_ingress_integrator_app_name | Name of the deployed NGINX Ingress Integrator application |
| nginx_ingress_integrator_requires |                                                           |
| nginx_ingress_integrator_provides |                                                           |

[terraform]: https://terraform.io
[hardware-api-charm]: https://charmhub.io/hardware-api
[juju-provider]: https://registry.terraform.io/providers/juju/juju/latest
[juju]: https://canonical.com/juju
[juju_model]: https://registry.terraform.io/providers/juju/juju/latest/docs/resources/model
[juju_application]: https://registry.terraform.io/providers/juju/juju/latest/docs/resources/application
[juju_integration]: https://registry.terraform.io/providers/juju/juju/latest/docs/resources/integration
