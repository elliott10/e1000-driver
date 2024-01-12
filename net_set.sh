#!/bin/bash

ip link set enp3s0 nomaster
ip link set eno1 master bridge0
ifup eno1

brctl show
