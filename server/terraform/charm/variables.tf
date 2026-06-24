variable "app_name" {
  type        = string
  default     = "api"
  description = "Name to give the deployed application"
}

variable "base" {
  type        = string
  nullable    = true
  default     = null
  description = "The operating system on which to deploy"
}

variable "channel" {
  type        = string
  default     = "latest/edge"
  description = "Channel of the charm"
}

variable "config" {
  type        = map(string)
  default     = {}
  description = "Map for config options"
}

variable "constraints" {
  type        = string
  nullable    = true
  default     = null
  description = "String listing constraints for this application"
}

variable "model_uuid" {
  type        = string
  description = "Reference to an existing model uuid"
}

variable "revision" {
  type        = number
  nullable    = true
  default     = null
  description = "Revision number of the charm"
}

variable "units" {
  type        = number
  default     = 1
  description = "Unit count/scale"
}
