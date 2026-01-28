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

resource "juju_application" "traefik_k8s" {
  name  = var.traefik_k8s.app_name
  model = data.juju_model.hardware_api.name
  trust = true
  charm {
    name     = "traefik-k8s"
    channel  = var.traefik_k8s.channel
    revision = var.traefik_k8s.revision
  }
  units  = 1
  config = var.traefik_k8s.config
}

resource "juju_integration" "traefik_k8s-hardware_api" {
  model = data.juju_model.hardware_api.name
  application {
    name     = juju_application.traefik_k8s.name
    endpoint = "traefik-route"
  }
  application {
    name     = module.hardware_api.app_name
    endpoint = module.hardware_api.endpoints.traefik_route
  }
}
