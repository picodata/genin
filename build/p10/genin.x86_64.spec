Name: genin
Version: ${VERSION}
Release: 1%{?dist}
Summary: Simple template engine that makes it easy to create inventory for tarantool cartridge
License: Public Domain
Group: Applications/File
Url: https://gitlab.com/picodata/devops/genin

%description
Simple template engine that makes it easy to create inventory for tarantool cartridge

%files
%attr(4755, root, root) %{_bindir}/%{name}
