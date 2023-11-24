 GENIN
---
![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/picodata/genin)
[![License](https://img.shields.io/badge/License-BSD_2--Clause-orange.svg)](LICENSE)
[![en](https://img.shields.io/badge/lang-en-cyan.svg)](https://github.com/picodata/genin/blob/master/README.md)
[![ru](https://img.shields.io/badge/lang-ru-green.svg)](https://github.com/picodata/genin/blob/master/README.ru.md)

- [GENIN](#genin)
    * [About](#about)
    * [Installation](#installation)
        - [RHEL, Fedora, Rockylinux](#rhel-centos-rockylinux-fedora)
        - [Ubuntu](#ubuntu)
        - [Debian](#debian)
        - [macOS](#macosx)
        - [Windows](#windows)
    * [Usage guide](#usage-guide)
        + [Inventory generation](#inventory-generation)
        + [Flags and options](#flags-and-options)
    * [Building from sources](#building-from-sources)
    * [Contributing](#contributing)
    * [Versioning](#versioning)
    * [Authors](#authors)
    * [License](#license)

---

## About
Genin is an inventory generator for `Ansible Cartridge`. It provides a command-line
tool that allows quick inventory creation for clusters of any size.
For example, an inventory file for a cluster of 50 replicasets can easily be of thousand
lines or more. Any slight change of the configuration, eg. adding a new configuration
option for all storages, means a lot of manual routine and increases the risk of
improper or incomplete configuration. Genin  allows you to stay confident while
maintaining the configuration file and steer clear of inaccuracies and human errors.
Genin is the tool that will help you very quickly roll out cluster configuration updates.

---

## Installation

Download and unzip the archive for the desired architecture.

---

#### Linux binary file

Universal executable:
```shell
curl -sLO https://binary.picodata.io/repository/raw/genin/bin/genin-0.5.8-x86_64-musl.tar.gz
tar -xvf genin-0.5.8-x86_64-musl.tar.gz ; sudo install genin /usr/local/bin/
```

---

#### RHEL, CentOS, Rockylinux, Fedora
There are two installation methods supported for RHEL, CentOS, Rockylinux and Fedora.

1. Installation using the package manager.

At first install picodata gpg key:
```shell
sudo rpm --import https://download.picodata.io/tarantool-picodata/el/RPM-GPG-KEY-kdy
```
Now add `picodata-release` repository.
RHEL 8.x, CentOS 8.x, Rockylinux 8.x, recent Fedora version
```shell
sudo yum install -y https://download.picodata.io/tarantool-picodata/el/8/x86_64/picodata-release-1.1.0.11-1.el8.x86_64.rpm
```
RHEL 7.x, CentOS 7.x
```shell
sudo yum install -y https://download.picodata.io/tarantool-picodata/el/7/x86_64/picodata-release-1.1.0.11-1.el7.x86_64.rpm
```
Update yum metadata.
```shell
sudo yum update
```
Install latest **Genin** package:
```shell
sudo yum install -y genin

> **Note:** with this installation method, all updates will
> also be available to you using `yum upgrade genin`

2. If you want to install `rpm` packages directly without
adding our repository.
```shell
sudo rpm -i https://binary.picodata.io/repository/yum/el/8/x86_64/os/genin-0.5.8-1.el8.x86_64.rpm
```
RHEL 7.x, CentOS 7.x
```shell
sudo rpm -i https://binary.picodata.io/repository/yum/el/7/x86_64/os/genin-0.5.8-1.el7.x86_64.rpm
```

---
> **Note:** please don't forget to pick the right package for your OS version.
---

####  Ubuntu

A `deb` package with `Genin` is available for installation on `ubuntu`.
Install package possible in two ways.

1. From the `picodata` repository:
Download and install gpg key:
```shell
wget -q -O - https://download.picodata.io/tarantool-picodata/ubuntu/picodata.gpg.key | sudo apt-key add -
```
Add `Picodata` repository:
```shell
sudo add-apt-repository 'deb [arch=amd64] https://download.picodata.io/tarantool-picodata/ubuntu/ focal main'
```
Install `Genin` package.
```shell
sudo apt install -y genin
```

2. Downloading and installing the package directly:
```shell
curl -sLO https://binary.picodata.io/repository/raw/genin/deb/genin-0.5.8.amd64.deb && sudo dpkg -i genin-0.5.8.amd64.deb
```

---

#### Debian

A `deb` package with `Genin` is available for installation on `debian`. Install package
possible in two ways.

1. From the `picodata` repository:
Download and install gpg key:
```shell
wget -q -O - https://download.picodata.io/tarantool-picodata/ubuntu/picodata.gpg.key | sudo apt-key add -
```
Add `Picodata` repository:
```shell
sudo add-apt-repository 'deb [arch=amd64] https://download.picodata.io/tarantool-picodata/debian/ bullseye main'
```
Install `Genin` package.
```shell
sudo apt install -y genin
```

2. Downloading and installing the package directly:
```shell
curl -sLO https://binary.picodata.io/repository/raw/genin/deb/genin-0.5.8.amd64.deb && sudo dpkg -i genin-0.5.8.amd64.deb
```

---

#### MacOSX
Installing with the `homebrew` package manager is the easiest way to
install Genin on MacOSX family (10.10+). If this is the first product of
`picodata` which you pay to use then you first need to add our `Tap`.
```shell
brew tap picodata/homebrew-tap
```
Now you can install Genin.
```shell
brew install genin
```

Use the following command to grab and install Genin in macOS (10.10+) wihtout
homebrew:
```shell
curl -L https://binary.picodata.io/repository/raw/genin/apple/genin-0.5.8-darwin-amd64.zip -o genin-0.5.8-darwin-amd64.zip
unzip genin-0.5.8-darwin-amd64.zip -d ~/bin/
```
> **Note:** The application can then be found under the `~/bin` directory.
> Make sure the directory is in your `$PATH`.

If you want to install a specific version, you can see the list of versions
using the command:
```shell
brew search genin
```
Install specific version:
```shell
brew install genin@0.3.8
```

#### Windows
Use the following command to grab and install Genin in Windows 7 64 bit or newer:
```shell
curl.exe -L https://binary.picodata.io/repository/raw/genin/windows/genin-0.5.8-darwin-amd64.zip -o genin-0.5.8-windows-amd64.zip
unzip.exe genin-0.5.8-windows-amd64.zip -d %HOME%/.cargo/bin/
```
> **Note:** The application can then be found under the `.cargo/bin` folder inside
> your user profile folder. Make sure it is in your `%PATH%`.

Сheck that the installation was successful:
```
genin --version
```

---
## Usage guide

### Inventory generation

First, let's generate a simple cluster for the `Vagrant` virtual environment.
For that `Genin` will need a `yaml` file with a concise list of cluster
details. That is a minimal cluster configuration file that features `Genin's`
own formatting. As long as users will likely need to have a descriptive
template of that file, `Genin` can automatically generate it with a built-in
dedicated subcommand:

```shell
genin init
```
This will result in creating the `cluster.genin.yaml` file in the current
directory.

> **Note:** If the `cluster.genin.yml` file already exists in current
> directory, then the new file will be named `cluster.genin.copy.yaml`.
> The `genin init` command will always append the `.copy` suffix to the
> file's name if the expected file exists.

Also, you can explicitly set the configuration file name:

```shell
genin init --output mycluster.yml
```
> **Note:** Use the `--output` flag together with the full path to
> `mycluster.yml` to specify the directory where the final cluster
> files will be saved.

Now you can open the file and examine the syntax.

```yaml
---
# list of replicasets as an array
topology:
  # List of replicasets as an array of arrays, where each element is a replicaset with a replica set
  - name: router              # (mandatory) replicaset name
    replicasets_count: 1      # (optional) how many masters we want, by default equal 1
    replication_factor: 0     # (optional) number of replicas in replicaset, default for router 0
    weight: 10                # (optional) replicaset weight
    zone:                     # (optional) zone parameter for ansible cartridge playbook
    roles:                    # (optional) list of roles
      - router
      - api
      - failover-coordinator
    config:                     # (optional) config with arbitrary key-values pairs
      replicaset_name: router   # any other configuration parameters in free order

  # all another replicasets generated using the init subcommand will have the same set of parameters
  - name: storage
    replicasets_count: 2        # this means 2 replicasets
    replication_factor: 3       # with 3 replicas in each replicaset
    weight: 10
    roles:
      - storage
    config:
      vshard_group: storage   # (optional) vshard group for vshard-storage
      all_rw: true            # (optional) all replicas can write data
      zone: tokio             # (optional) zone for ansible cartridge

# map of regions, datacenters, and hosts
hosts:
  - name: cloud           # (mandatory) hostname or domain name
                          # in this example, both hosts are in the same cloud data center
    config:               # (optional) begin binary and http port, by default 8081, 3031
                          # ports can be defined on all levels
      http_port: 8081          # (optional) http port to start counting from
      binary_port: 3031        # (optional) binary port to start counting from
    hosts:
      - name: host-1      # (mandatory) hostname or domain name
        config:
          address: 192.168.16.11  # address can be IP, url, subnet (subnet allowed only for higher levels)
      - name: host-2
        config:
          address: host-1.cloud.com

# failover parameters
failover:
  mode: stateful               # (optional) failover mode (stateful, eventual, disabled)
  state_provider: stateboard   # (optional) what is serve failover (stateboard, stateful)
  failover_timeout: 60         # (optional) timeout (in seconds), used by membership to mark `suspect` members as `dead`
  fencing_enabled: true        # (optional) abandon leadership when both the state provider quorum and at least one
                               # replica are lost (suitable in stateful mode only)
  fencing_timeout: 20          # (optional) time (in seconds) to actuate fencing after the check fails
  fencing_pause: 120           # (optional) the period (in seconds) of performing the check
  stateboard_params:                  # (optional) params for chosen in state_provider failover type
      uri: 192.168.16.1:4401
      password: "vG?-GG!4sxV8q5:f"

# vars similar to those configured in the TDG inventory
vars:
  ansible_user: my_user
  ansible_password: my_user_password
  cartridge_app_name: my_app
  cartridge_package_path: /tmp/my_app.rpm
  cartridge_cluster_cookie: my_app_cluster_cookie
  # put here you personally key/value ansible cartridge vars
```

Replace the stubs with the actual values of your hosts and their parameters
and save the file.

So far you are already halfway through getting things done! Use the resulted
`Genin` configuration file to generate the final inventory file. Here is the
required command for that:

```shell
genin build
```

Done! The `inventory.yaml` file will appear in the same directory where you
launched `Genin`. Now we can set up the cluster:

```shell
ansible-playbook -i inventory.yaml playbook.yaml
```

---

### Editing the cluster configuration

---

#### Minimal configuration

The initial cluster configuration file can be slimmed down to the following
minimal variant:

```yaml
---
topology:
  - name: router            # since replicasets_count is not set for this replicaset,
                            # and also no roles are set, then the genin will automatically determine it by name as
                            # router and set the number of replicas in the replication set to 1 (replication count: 1)
  - name: storage
    replicasets_count: 3
    replication_factor: 2

hosts:
  - name: cloud
    config:
      address: 192.168.12/32
    hosts:
      - name: host-1        # ip 192.168.16.12 will be automatically allocated from the above subnet
      - name: host-2        # ip 192.168.16.13 will be automatically allocated from the above subnet
```

This is a perfectly valid and working configuration file. The rest of the
parameters wil use their default values.

A set of examples can be found in the directory [docs/examples](docs/examples).

---

#### Changing the Topology

---

Now let's change the file, and expand our cluster to make the inventory more
similar to the real one. To do this, we will increase the number of routers of
hosts, routers, stacks up to 10. Change the number of storage replicas to 1,
and add a custom replicaset `cache` in the amount of 5 pieces.

```yaml
---
topology:
  - name: router
    replication_factor: 10    # this replicaset has no roles defined and its name is a router,
                              # so the replicasets_count parameter will be ignored
                              # and the default number of replicasets will be set to 1
  - name: storage
    replicasets_count: 10     # since the number of replicases in replicaset is not set,
                              # the default will be 10 replicasets with 1 replica in each

hosts:
  - name: cloud
    hosts:
      - name: host-1
        config:
          address: 192.168.16.11      # in this example, the address for each host is set separately,
                                      # but for convenience, the address could be set by subnet,
                                      # specifying it one level higher for cloud
      - name: host-2
        config:
          address: 192.168.16.12
      - name: host-3
        config:
          address: 192.168.16.13
      - name: host-4
        config:
          address: 192.168.16.14
      - name: host-5
        config:
          address: 192.168.16.15
      - name: host-6
        config:
          address: 192.168.16.16
      - name: host-7
        config:
          address: 192.168.16.17
      - name: host-8
        config:
          address: 192.168.16.18
      - name: host-9
        config:
          address: 192.168.16.19
      - name: host-10
        config:
          address: 192.168.16.20
```
The actual difference between the 2 replicasets configuration and a large
cluster configuration is not that great, whereas the resulting inventory file
for the large cluster will be 5 times bigger.

---

#### Redefining failover

---

Until now, our cluster has always been with `stateful failover` , now let's
change it to `disabled` and run the generation with the command:

```shell
genin build --failover-state-provider etcd2
```
> **Note:** All options have a short version, for example for provider
> failover is `genin build -F etcd2`

This flag will override the failover type we specified in `cluster.genin.yaml`,
and add default values based on those recommended for `etcd2`.

The options related to failover and other subcommands work in the same way.
```shell
genin init --failover-mode disabled
```
`disabled` argument for `failover-mode` and `failover-state-provider` disable
failover.

You can learn more about the operation of the failover and the possible options
in the documentation. [Tarantool documentation](https://www.tarantool.io/ru/doc/1.10/book/cartridge/topics/failover/).

---

#### Balancing and distribution control

By default, `Genin` always allocates instances based on the current host load.
For example the following configuration for a genin would be distributed in
the following way:

```yaml
---
topology:
  - name: router
    replicasets_count: 4
    roles:
      - router
      - failover-coordinator
  - name: storage
    replicasets_count: 2
    replication_factor: 4
    roles:
      - storage
  - name: cache
    replicasets_count: 2
    roles:
      - cache
hosts:
  - name: dc-1
    hosts:
      - name: server-1
        config:
          address: 192.168.16.11
      - name: server-2
        config:
          address: 192.168.16.12
  - name: dc-2
    hosts:
      - name: server-3
        config:
          address: 192.168.16.13
      - name: server-4
        config:
          address: 192.168.16.14
...
```

![default-spreading](docs/images/default-spreading.gif)

Although for most automatic distribution clusters `Genin` should be enough,
there can always be a situation where some replicasets must be allocated to
a specific host or group of hosts. For this there is a special option
`failure_domains` in the `Genin` configuration.

```yaml
---
topology:
  - name: router
    replicasets_count: 4
    roles:
      - router
      - failover-coordinator
  - name: storage
    replicasets_count: 2
    replication_factor: 4
    roles:
      - storage
  - name: cache
    replicasets_count: 2
    failure_domains: [dc-2]
    roles:
      - cache
hosts:
  - name: dc-1
    hosts:
      - name: server-1
        config:
          address: 192.168.16.11
      - name: server-2
        config:
          address: 192.168.16.12
  - name: dc-2
    hosts:
      - name: server-3
        config:
          address: 192.168.16.13
      - name: server-4
        config:
          address: 192.168.16.14
...
```
This will cause Genin to allocate the cache replicaset as follows:

![failure-domains-1](docs/images/failure-domains-1.gif)

If you specify several host names in `failure_domains` at once
`[ server-2, server-3 ]`, then `cache` replicaset instances are guaranteed
will be on the specified hosts.

![failure-domains-2](docs/images/failure-domains-2.gif)

---
#### Use failure domain name as a zone for the instance config

You can also use a failure domain name as a value for the [`zone`](https://github.com/tarantool/ansible-cartridge/blob/master/doc/variables.md?plain=1#L90) property of the instances config. Just add the `--fd-as-zone` flag to your `build` command, for example: `genin build --fd-as-zone`.

With this flag, once the instances are distributed over the target hosts according to the failure_domains algorithm, the final host name of each instance becomes its zone parameter and gets stored in the instance's config.

---

#### Configuration `ansible-host`

The `ansible-host` variable needs to be set in the final inventory configuration at the hosts. An example of a desirable one:

```yaml
storages:
  vars:
    ansible_host: 192.168.1.1
  hosts:
    storage-1-1: ~
```

By default, `ansible_host` is populated from the `address` parameter of the cluster host configuration,
for example for the above result the source might look like this:

```yaml
hosts:
  - name: storages
    config:
      address: 192.168.1.1
```

You can also set this parameter manually via the `ansible_host` host configuration parameter. It has a higher priority:

```yaml
hosts:
  - name: storages
    config:
      address: 192.168.1.1
      ansible_host: 192.168.1.2
```

---

### Reverse parsing config

Since `Genin` is a relatively new tool, and `picodata` is far from full
`tarantool` clusters were generated with it, then to simplify transition
there is a `reverse` command.

```shell
genin reverse -s inventory.yml -o cluster.genin.yml
```

This command allows you to parse the already finished inventory and get the
configuration `cluster.genin.yml` for `genin build`. Why might this be needed?
For example for the `upgrade` command. which serves to add new replicasets or
instances and requires 2 configuration files.

The `reverse` command has several important features:
1. Since the inventory is a flat list, without division by domains, data
centers, regions, then `genin` cannot know anything about the structure and
will generate a flat configuration too.

For example, if we go through a loop, then we will lose all information about
`failure` domains:
```shell
genin init -o cluster.genin.yml
genin build -s cluster.genin.yml -o inventory.yml
genin reverse -o inventory.yml
```

The last command, the `reserve` call, will display the table without domains.
```text
+-------------+-------------+
|          cluster          |
+-------------+-------------+
|  server-1   |  server-2   |
+-------------+-------------+
| router-1-1  | storage-1-1 |
| 8081 8081   | 8081 8081   |
+-------------+-------------+
| storage-1-2 | storage-2-1 |
| 8082 8082   | 8082 8082   |
+-------------+-------------+
| storage-2-2 |             |
| 8083 8083   |             |
+-------------+-------------+
```

2. The configuration file received in using `reverse` will also have
slightly different configuration.

```yaml
---
topology:
  - name: router
    replicasets_count: 1
    replication_factor: 1
    roles:
      - router
      - failover-coordinator
    config:
      http_port: 8081
      binary_port: 8081
      address: 192.168.16.11
  - name: storage
    replicasets_count: 2
    replication_factor: 2
    roles:
      - storage
    config:
      http_port: 8082
      binary_port: 8082
      address: 192.168.16.11
hosts:
  - name: server-1
    config:
      address: 192.168.16.11
  - name: server-2
    config:
      address: 192.168.16.12
failover:
  mode: stateful
  state_provider: stateboard
  stateboard_params:
    uri: "192.168.16.11:4401"
    password: password
vars:
  ansible_user: ansible
  ansible_password: ansible
  cartridge_app_name: myapp
  cartridge_cluster_cookie: myapp-cookie
  cartridge_package_path: /tmp/myapp.rpm
  cartridge_bootstrap_vshard: true
```

#### Cluster reconfiguration

To update a deployed cluster using the generated `Genin` inventory, there is
a special `upgrade` command for adding new instances. Unlike inventory
regeneration, when which all instances are redistributed each time anew,
`upgrade` will leave distribution as is, and will only add new instances.
This will allow painlessly upgrade the cluster, without a full redeploy.

To run `upgrade` you need to pass two required arguments `--old`
and `--new`.
```shell
genin upgrade --old cluster.genin.yml --new upgrade.genin.yml -o inventory.yml
```

The `--old` option specifies the path to the old cluster config we want
to do an `upgrade`.
Option `--new` path to new cluster configuration based on which `Genin`
will make `diff` and add those instances that were not in the config passed to
`--old`.

| Old                                                                       | New                                                                       | Diff                                                          |
|---------------------------------------------------------------------------|---------------------------------------------------------------------------|---------------------------------------------------------------|
| <pre>- name: router<br/>  replicasets_count: 2                            | <pre>- name: router<br>  replicasets_count: 4                             | <pre>router-3<br>router-4                                     |
| <pre>- name: storage<br>  replicasets_count: 2<br>  replication_factor: 2 | <pre>- name: storage<br>  replicasets_count: 2<br>  replication_factor: 4 | <pre>storage-1-3<br>storage-1-4<br>storage-2-3<br>storage-2-4 |

---

> **Note:** currently, cluster `downgrade` is only partially implemented and
> requires manual verification of the resulting inventory.

---

### Flags and options

Here we describe few other useful flags and options that you might want to
use with `genin`.
First, you can always change paths of both source and target configuration
files using the `--source` (short `-s`) and `--output` (short `-o`) flags
respectively. We recommend using the `.genin.` suffix for naming convenience.
Here is the example command in two variants:
```shell
genin init --output /home/tarantool/custom_cluster_name.yml
genin show --source /home/tarantool/custom_cluster_name.yml
genin build --source /home/user/path/my_cluster.genin.yml --output /home/tarantool/cluster-new/my_hosts.yml
genin build -s /home/user/path/my_cluster.genin.yml -o /home/tarantool/cluster-new/my_hosts.yml
```
> **Note:** It is worth reiterating that if any of the subcommands
> `build`, `init` etc. `Genin` will look for files with default names
> (such as cluster.genin.yaml, inventory.yaml)

Next, there is a very useful option for controlling the log output. There
are three supported log levels that you can enable using the `-v` flag with
a desired number of extra `v` letters for more verbosity (up to`-vvv`).
Single `v` means *INFO*, double means *DEBUG*, and three or more mean *TRACE*.
```
genin build -vvv
```

---

> **Note** The number of displayed information increases with the addition of new
> occurrences of `-v`. For example `-vv` will be equivalent to *TRACE* level

---

There is an option `--scheme` to control how the schema is displayed (short
option -p) which accepts a sequence of equivalent characters display options.

// Character table under development

In addition, flags exist to convey sensitive information.

```shell
genin build --ansible-user dmitry.travyan --ansible-password ddfqd
genin build --cartridge-cluster-cookie R68sJfV4C2hLrWC3
```

As mentioned earlier, by default, `Genin` will create copies of files if
The target file already exists in the specified path. In order to force
overwrite the target file, there is a `--force` flag (or short `-f`).
```shell
genin build -o my-cluster.yml
genin build -o my-cluster.yml --force
```

## Building from sources

At first, you need to clone the source code.
```shell
git clone https://github.com/picodata/genin.git
```

Second, you need to install Rust build tools.
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---
> **Note:** You should refresh the `$PATH` variable to get access to locally installed `rust binaries`.
---

After installing all required tools it is time to build and install `genin`.
```shell
cd genin
rustup override set nightly
cargo +nightly build --release
install -m 001 target/release/genin /usr/local/bin/
```

---
> **Note:** Do not forget to install build tools and dependencies before building `genin`.
>
> For Debian-based distributions:
> ```shell
> sudo apt install -y build-essential
> ```
> For Red Hat-based distributions (RHEL, CentOS, Fedora):
> ```
> sudo yum install -y gcc
> ```
> For macOS make sure you have `Command Line Developer Tools` installed (`xcode-select --install`).
---

Check that the installation was successful:
```shell
genin --version
```

## Contributing

Pull requests are welcome. For major changes, please open an issue first to
discuss what you would like to change.

Please make sure to update tests as appropriate.

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available,
see the [tags on this repository](https://github.com/picodata/genin/tags).

## Authors

- **Dmitry Travyan**
- **Lomakina Anastasia**

© 2020-2023 Picodata.io https://github.com/picodata

## License

This project is licensed under the BSD License - see the [LICENSE](LICENSE) file for details.
