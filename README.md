# data-flow-rs

[![Build](https://github.com/oliverjantar/data-flow-rs/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/oliverjantar/data-flow-rs/actions/workflows/ci.yml)

Simple download pipeline for EVM chains.

Old blocks are firstly downloaded with http provider and then switched to web socket for subscribing for new blocks.

Full blocks are sent to channel for further processing.