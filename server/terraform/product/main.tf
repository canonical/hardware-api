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

resource "juju_application" "nginx_ingress_integrator" {
  name  = var.nginx_ingress_integrator.app_name
  model = data.juju_model.hardware_api.name
  trust = true
  charm {
    name     = "nginx-ingress-integrator"
    channel  = var.nginx_ingress_integrator.channel
    revision = var.nginx_ingress_integrator.revision
  }
  units  = var.nginx_ingress_integrator.units
  config = var.nginx_ingress_integrator.config
}

resource "juju_integration" "hardware_api_ingress" {
  model = data.juju_model.hardware_api.name
  application {
    name     = module.hardware_api.app_name
    endpoint = module.hardware_api.endpoints.nginx_route
  }
  application {
    name     = juju_application.nginx_ingress_integrator.name
    endpoint = "nginx-route"
  }
}
