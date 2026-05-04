# \ScriptsApi

All URIs are relative to *http://localhost:1234*

Method | HTTP request | Description
------------- | ------------- | -------------
[**scripts_get**](ScriptsApi.md#scripts_get) | **GET** /scripts | 
[**scripts_post**](ScriptsApi.md#scripts_post) | **POST** /scripts | 
[**scripts_script_id_delete**](ScriptsApi.md#scripts_script_id_delete) | **DELETE** /scripts/{scriptId} | 
[**scripts_script_id_get**](ScriptsApi.md#scripts_script_id_get) | **GET** /scripts/{scriptId} | 
[**scripts_script_id_put**](ScriptsApi.md#scripts_script_id_put) | **PUT** /scripts/{scriptId} | 



## scripts_get

> Vec<models::Script> scripts_get()


### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::Script>**](Script.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## scripts_post

> scripts_post(scripts_post_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**scripts_post_request** | [**ScriptsPostRequest**](ScriptsPostRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## scripts_script_id_delete

> scripts_script_id_delete(script_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**script_id** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## scripts_script_id_get

> models::Script scripts_script_id_get(script_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**script_id** | **String** |  | [required] |

### Return type

[**models::Script**](Script.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## scripts_script_id_put

> scripts_script_id_put(script_id, scripts_script_id_put_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**script_id** | **String** |  | [required] |
**scripts_script_id_put_request** | [**ScriptsScriptIdPutRequest**](ScriptsScriptIdPutRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

