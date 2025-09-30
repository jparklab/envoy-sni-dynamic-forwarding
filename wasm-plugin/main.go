package main

import (
	"github.com/proxy-wasm/proxy-wasm-go-sdk/proxywasm"
	"github.com/proxy-wasm/proxy-wasm-go-sdk/proxywasm/types"
)

func main() {

}

func init() {
	proxywasm.SetVMContext(&vmContext{})
}

type vmContext struct {
	types.DefaultVMContext
}

func (*vmContext) NewPluginContext(contextID uint32) types.PluginContext {
	return &pluginContext{}
}

type pluginContext struct {
	types.DefaultPluginContext
}

func (*pluginContext) NewTcpContext(contextID uint32) types.TcpContext {
	return &tcpContext{}
}

type tcpContext struct {
	types.DefaultTcpContext
}

func (*tcpContext) dumpAndSetProperty() {
	var err error

	paths := [][]string{
		/* works (can be used to dump all metadata)
		{
			"xds",
			"listener_metadata",
			"filter_metadata",
		},
		{
			"metadata",
		},
		*/
		{
			"metadata",
			"filter_metadata",
			"envoy.filters.network.ext_authz",
			"envoy.upstream.dynamic_port",
		},
		/* works when it was set by the set filter state filter
		{
			"filter_state",
			"envoy.network.upstream_server_name",
		},
		*/
	}

	proxywasm.LogInfof("list properties")
	for _, path := range paths {
		value, err := proxywasm.GetProperty(path)
		if err != nil {
			proxywasm.LogInfof("failed to get property for %v: %s", path, err)
		} else {
			proxywasm.LogInfof("value for %v: %s", path, string(value))
		}
	}

	dynamic_host_port := []string{
		"metadata",
		"filter_metadata",
		"envoy.filters.network.ext_authz",
		"envoy.upstream.dynamic_port",
	}

	port_value, err := proxywasm.GetProperty(dynamic_host_port)
	if err != nil {
		proxywasm.LogInfof("failed to get property for %v: %s", dynamic_host_port, err)
		return
	} else {
		proxywasm.LogInfof("value for %v: %s", dynamic_host_port, string(port_value))
	}

	/* // we cannot use SetProperty since it adds 'wasm' prefix https://github.com/envoyproxy/envoy/issues/28673
		var err error
		err = proxywasm.SetProperty(
			[]string{
				"filter_state",
				// "on_new_connection",
				"envoy.upstream.dynamic_port",
			},
			[]byte(value),
		)
		if err != nil {
			proxywasm.LogInfof("failed to set property: %s", err)
		} else {
			proxywasm.LogInfof("set property: %s", string(value))
		}
	}
	*/

	// /home/jpark/github/envoyproxy/envoy/source/extensions/common/wasm/ext/set_envoy_filter_state.proto
	{
		// encoded protobuf
		key := []byte("envoy.upstream.dynamic_port")
		encoded := make([]byte, 0)
		encoded = append(encoded, []byte{(1 << 3) | 2, byte(len(key))}...)
		encoded = append(encoded, key...)
		encoded = append(encoded, []byte{(2 << 3) | 2, byte(len(port_value))}...)
		encoded = append(encoded, port_value...)
		encoded = append(encoded, []byte{(3 << 3), 2}...)

		ret, err := proxywasm.CallForeignFunction(
			"set_envoy_filter_state",
			encoded,
		)
		if err != nil {
			proxywasm.LogInfof("failed to call set_envoy_filter_state: %s", err)
		} else {
			proxywasm.LogInfof("set set_envoy_filter_state: %s", string(ret))
		}
	}
	{
		// encoded protobuf
		key := []byte("envoy.upstream.dynamic_host")
		addr_value := []byte("127.0.0.1")
		encoded := make([]byte, 0)
		encoded = append(encoded, []byte{(1 << 3) | 2, byte(len(key))}...)
		encoded = append(encoded, key...)
		encoded = append(encoded, []byte{(2 << 3) | 2, byte(len(addr_value))}...)
		encoded = append(encoded, addr_value...)
		encoded = append(encoded, []byte{(3 << 3), 2}...)

		ret, err := proxywasm.CallForeignFunction(
			"set_envoy_filter_state",
			encoded,
		)
		if err != nil {
			proxywasm.LogInfof("failed to call set_envoy_filter_state: %s", err)
		} else {
			proxywasm.LogInfof("set set_envoy_filter_state: %s", string(ret))
		}
	}

	/* doesn't work since protobuf is not supported in tinygo
	  https://github.com/tetratelabs/proxy-wasm-go-sdk/issues/324
	setFilterStateConfig := sfs.Config{
		OnNewConnection: []*sfs_common.FilterStateValue{
			{
				Key: &sfs_common.FilterStateValue_ObjectKey{
					ObjectKey: "envoy.upstream.dynamic_host",
				},
				Value: &sfs_common.FilterStateValue_FormatString{
					FormatString: &core.SubstitutionFormatString{
						Format: &core.SubstitutionFormatString_TextFormat{
							TextFormat: "127.0.0.1",
						},
					},
				},
			},
		},
	}
	encoded, err := proto.Marshal(&setFilterStateConfig)
	if err != nil {
		proxywasm.LogInfof("failed to marshal set filter state config: %s", err)
	} else {
	}
	*/
}

func (c *tcpContext) OnDownstreamData(dataSize int, endOfStream bool) types.Action {
	return types.ActionContinue
}

func (c *tcpContext) OnNewConnection() types.Action {
	c.dumpAndSetProperty()

	// proxywasm.LogInfo("Hello, world!")

	return types.ActionContinue
}
