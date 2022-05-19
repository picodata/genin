terraform {
  required_version = ">= 0.14.0"
    required_providers {
      openstack = {
        source  = "terraform-provider-openstack/openstack"
        version = "~> 1.46.0"
      }
      selectel = {
        source  = "selectel/selectel"
        version = "~> 3.7.1"
      }
   }
}

provider "selectel" {
  token = var.sel_token
  region = var.os_region
}

provider "openstack" {
  domain_name = var.sel_account
  user_name   = var.user_name
  password    = var.user_password
  auth_url    = var.os_auth_url
  tenant_id   = var.project_id
  region      = var.os_region
}

resource openstack_compute_keypair_v2 "dtravyan" {
  name = "dtravyan"
}

data "openstack_networking_network_v2" "external_net" {
  name = "external-network"
}

resource "openstack_networking_router_v2" "genin_router" {
  name = "genin_router"
  external_network_id = data.openstack_networking_network_v2.external_net.id
}

resource "openstack_networking_network_v2" "genin_network" {
  name = "genin_network"
}

resource "openstack_networking_subnet_v2" "genin_subnet" {
  network_id = openstack_networking_network_v2.genin_network.id
  name       = "genin_subnet"
  cidr       = var.subnet_cidr
  enable_dhcp = false
  dns_nameservers = ["8.8.8.8"]
}

resource "openstack_networking_router_interface_v2" "genin_router_interface" {
  router_id = openstack_networking_router_v2.genin_router.id
  subnet_id = openstack_networking_subnet_v2.genin_subnet.id
}

data "openstack_images_image_v2" "centos_image" {
  most_recent = true
  visibility  = "public"
  name        = "CentOS 7 Minimal 64-bit"
}

data "cloudinit_config" "cloud_init_bastion" {
  gzip = false
  base64_encode = false
  part {
    content_type = "text/cloud-config" 
    content = file("${path.module}/cloud-init/cloud-init-bastion.yml")
  }
}

resource "openstack_compute_flavor_v2" "genin_bastion_flavor" {
  name      = "bastion-flavour"
  ram       = var.genin_hosts_ram_mb
  vcpus     = var.genin_hosts_vcpus
  disk      = var.genin_hosts_root_disk_gb
  is_public = "false"
}

resource "openstack_compute_instance_v2" "genin_host" {
  count             = var.genin_hosts_count
  name              = "genin_0${1+count.index}_host"
  flavor_id         = openstack_compute_flavor_v2.genin_bastion_flavor.id
  key_pair          = openstack_compute_keypair_v2.dtravyan.id
  availability_zone = var.server_zone
  network {
    fixed_ip_v4 = "192.168.16.${11+count.index}"
    uuid = openstack_networking_network_v2.genin_network.id
  }

  image_id = data.openstack_images_image_v2.centos_image.id

  vendor_options {
    ignore_resize_confirmation = true
  }

  lifecycle {
    ignore_changes = [image_id]
  }

  user_data = data.cloudinit_config.cloud_init_bastion.rendered
}

resource "openstack_networking_floatingip_v2" "genin_floating_it" {
  pool = "external-network"
}

resource "openstack_compute_floatingip_associate_v2" "genin_floating_it" {
  count = var.genin_hosts_count
  floating_ip = openstack_networking_floatingip_v2.genin_floating_it.address
  instance_id = "openstack_compute_instance_v2.genin_0${1+count.index}_trn_host.id"
}
