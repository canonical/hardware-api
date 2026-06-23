data "juju_model" "hardware_api" {
  name  = var.model
  owner = var.owner
}

module "hardware_api" {
  source      = "../charm"
  app_name    = var.hardware_api.app_name
  channel     = var.hardware_api.channel
  config      = var.hardware_api.config
  model_uuid  = data.juju_model.hardware_api.uuid
  constraints = var.hardware_api.constraints
  revision    = var.hardware_api.revision
  base        = var.hardware_api.base
  units       = var.hardware_api.units
}

resource "juju_application" "ingress_configurator" {
  name       = var.ingress_configurator.app_name
  model_uuid = data.juju_model.hardware_api.uuid
  trust      = true
  charm {
    name     = "ingress-configurator"
    channel  = var.ingress_configurator.channel
    revision = var.ingress_configurator.revision
  }
  units  = var.ingress_configurator.units
  config = var.ingress_configurator.config
}

resource "juju_integration" "hardware_api_ingress" {
  model_uuid = data.juju_model.hardware_api.uuid
  application {
    name     = module.hardware_api.application.name
    endpoint = module.hardware_api.requires.ingress
  }
  application {
    name     = juju_application.ingress_configurator.name
    endpoint = "ingress"
  }
}

resource "juju_integration" "ingress_configurator_haproxy" {
  count      = var.haproxy_offer != null ? 1 : 0
  model_uuid = data.juju_model.hardware_api.uuid
  application {
    name     = juju_application.ingress_configurator.name
    endpoint = "haproxy-route"
  }
  application {
    offer_url = var.haproxy_offer
  }
}
