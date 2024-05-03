terraform {
  required_providers {
    juju = {
      version = "~> 0.10.1"
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
    hostname = var.external_ingress_hostname
    port     = var.environment == "development" ? 30000 : 443
  }

  units = 1
}

resource "juju_application" "ingress" {
  name  = "ingress"
  model = juju_model.hardware-api.name
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
  model = juju_model.hardware-api.name

  application {
    name = juju_application.hardware-api.name
  }

  application {
    name = juju_application.ingress.name
  }
}
