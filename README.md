# data-flow-rs

[![Build](https://github.com/oliverjantar/data-flow-rs/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/oliverjantar/data-flow-rs/actions/workflows/ci.yml)

Simple download pipeline for EVM chains.

Firstly, history of blocks is downloaded using http provider and simple get_blocks one by one. Once blocks are synced to tip, download is switched to a web socket to subscribe for new blocks.

Full blocks with transactions are sent to channel for further processing.