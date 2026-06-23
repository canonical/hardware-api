resource "juju_application" "hardware_api" {
  name        = var.app_name
  model_uuid  = var.model_uuid
  constraints = var.constraints
  config      = var.config
  units       = var.units


  charm {
    name     = "hardware-api"
    base     = var.base
    channel  = var.channel
    revision = var.revision
  }
}
