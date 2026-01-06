data "juju_model" "hardware_api" {
  name = var.model
}

module "hardware_api" {
  source      = "../charm"
  app_name    = var.hardware_api.app_name
  channel     = var.hardware_api.channel
  config      = var.hardware_api.config
  model       = data.juju_model.hardware_api.name
  constraints = var.hardware_api.constraints
  revision    = var.hardware_api.revision
  base        = var.hardware_api.base
  units       = var.hardware_api.units
}

resource "juju_application" "traefik" {
  name  = var.traefik.app_name
  model = data.juju_model.hardware_api.name
  trust = true
  charm {
    name     = "traefik-k8s"
    channel  = var.traefik.channel
    revision = var.traefik.revision
  }
  units  = var.traefik.units
  config = var.traefik.config
}

resource "juju_application" "lego" {
  name  = var.lego.app_name
  model = data.juju_model.hardware_api.name
  charm {
    name     = "lego"
    channel  = var.lego.channel
    revision = var.lego.revision
  }
  units  = var.lego.units
  config = var.lego.config
}

resource "juju_integration" "hardware_api_ingress" {
  model = data.juju_model.hardware_api.name
  application {
    name     = module.hardware_api.app_name
    endpoint = module.hardware_api.endpoints.ingress
  }
  application {
    name     = juju_application.traefik.name
    endpoint = "ingress"
  }
}

resource "juju_integration" "traefik_certificates" {
  model = data.juju_model.hardware_api.name
  application {
    name     = juju_application.traefik.name
    endpoint = "certificates"
  }
  application {
    name     = juju_application.lego.name
    endpoint = "certificates"
  }
}
