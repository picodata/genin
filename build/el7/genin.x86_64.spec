Name: genin
Version: ${VERSION}
Release: 1%{?dist}
Summary: Simple template engine that makes it easy to create inventory for tarantool cartridge
License: Public Domain
URL: https://gitlab.com/picodata/devops/genin

%description
Simple template engine that makes it easy to create inventory for tarantool cartridge

%prep
rm -rf $RPM_BUILD_ROOT


%install
mkdir -p $RPM_BUILD_ROOT/bin ~/bin
mv $GITHUB_WORKSPACE/x86_64-unknown-linux-musl/genin $RPM_BUILD_ROOT/bin/genin
install $RPM_BUILD_ROOT/bin/%{name} /usr/local/bin/%{name}
chmod +x /usr/local/bin/%{name}

%files
/bin/%{name}
