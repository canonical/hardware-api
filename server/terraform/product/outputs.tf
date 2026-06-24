output "models" {
  description = "Map of the key of the model and the components deployed in the model"
  value = {
    hardware_api = {
      model_uuid = data.juju_model.hardware_api.uuid
      components = {
        hardware_api         = module.hardware_api.application
        ingress_configurator = juju_application.ingress_configurator
      }
    }
  }
}
