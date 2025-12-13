resource "juju_application" "hardware_api" {
  name  = var.app_name
  model = var.model

  charm {
    name     = "hardware-api"
    base     = var.base
    channel  = var.channel
    revision = var.revision
  }

  config      = var.config
  constraints = var.constraints
  units       = var.units
}
