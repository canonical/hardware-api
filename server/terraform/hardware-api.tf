terraform {
  required_providers {
    juju = {
      version = "~> 0.11.0"
      source  = "juju/juju"
    }
  }
}

provider "juju" {}

variable "environment" {
  description = "The environment to deploy to (development, staging, production)"
}

variable "external_ingress_hostname" {
  description = "External hostname for the ingress"
  type        = string
  default     = "hardware-api.ubuntu.com"
}

resource "juju_model" "hardware-api" {
  name = "hardware-api-${var.environment}"
}

resource "juju_application" "hardware-api" {
  name  = "api"
  model = juju_model.hardware-api.name

  charm {
    name    = "hardware-api"
    channel = "latest/edge"
  }

  config = {
    hostname = var.environment == "staging" ? "hardware-api-staging.${var.external_ingress_hostname}" : "hardware-api.${var.external_ingress_hostname}"
    port     = var.environment == "development" ? 30000 : 443
  }

  units = 1
}

# resource "juju_integration" "hardware-api-ingress" {
#   model = juju_model.hardware-api.name
#
#   application {
#     name = juju_application.hardware-api.name
#   }
#
#   application {
#     name = juju_application.ingress.name
#   }
# }
