terraform {
  required_providers {
    juju = {
      version = "~> 0.19.0"
      source  = "juju/juju"
    }
  }
}

variable "environment" {
  description = "The environment to deploy to (development, staging, production)"
}

variable "tls_secret_name" {
  description = "Secret where the TLS certificate for ingress is stored"
  type        = string
  default     = ""
}

variable "external_ingress_hostname" {
  description = "External hostname for the ingress"
  type        = string
  default     = "hw.ubuntu.com"
}

locals {
  app_model = "hardware-api-${var.environment}"
}

resource "juju_application" "hardware-api" {
  name  = "api"
  model = local.app_model

  charm {
    name    = "hardware-api"
    channel = "latest/edge"
  }

  config = {
    hostname = var.external_ingress_hostname
    port     = var.environment == "development" ? 30000 : 443
  }

  units = 1
}

resource "juju_application" "ingress" {
  name  = "ingress"
  model = local.app_model
  trust = true

  charm {
    name    = "nginx-ingress-integrator"
    channel = "latest/stable"
  }

  config = {
    tls-secret-name = var.tls_secret_name
  }
}


resource "juju_integration" "hardware-api-ingress" {
  model = local.app_model

  application {
    name = juju_application.hardware-api.name
  }

  application {
    name = juju_application.ingress.name
  }
}
