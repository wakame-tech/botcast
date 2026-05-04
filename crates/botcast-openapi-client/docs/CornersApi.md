# \CornersApi

All URIs are relative to *http://localhost:1234*

Method | HTTP request | Description
------------- | ------------- | -------------
[**corners_corner_id_delete**](CornersApi.md#corners_corner_id_delete) | **DELETE** /corners/{cornerId} | 
[**corners_corner_id_get**](CornersApi.md#corners_corner_id_get) | **GET** /corners/{cornerId} | 
[**corners_corner_id_put**](CornersApi.md#corners_corner_id_put) | **PUT** /corners/{cornerId} | 



## corners_corner_id_delete

> corners_corner_id_delete(corner_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**corner_id** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## corners_corner_id_get

> models::Corner corners_corner_id_get(corner_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**corner_id** | **String** |  | [required] |

### Return type

[**models::Corner**](Corner.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## corners_corner_id_put

> corners_corner_id_put(corner_id, corners_corner_id_put_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**corner_id** | **String** |  | [required] |
**corners_corner_id_put_request** | [**CornersCornerIdPutRequest**](CornersCornerIdPutRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

