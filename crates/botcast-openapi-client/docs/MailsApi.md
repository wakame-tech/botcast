# \MailsApi

All URIs are relative to *http://localhost:1234*

Method | HTTP request | Description
------------- | ------------- | -------------
[**corners_corner_id_mails_get**](MailsApi.md#corners_corner_id_mails_get) | **GET** /corners/{cornerId}/mails | 
[**corners_corner_id_mails_mail_id_delete**](MailsApi.md#corners_corner_id_mails_mail_id_delete) | **DELETE** /corners/{cornerId}/mails/{mailId} | 
[**corners_corner_id_mails_post**](MailsApi.md#corners_corner_id_mails_post) | **POST** /corners/{cornerId}/mails | 



## corners_corner_id_mails_get

> Vec<models::Mail> corners_corner_id_mails_get(corner_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**corner_id** | **String** |  | [required] |

### Return type

[**Vec<models::Mail>**](Mail.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## corners_corner_id_mails_mail_id_delete

> corners_corner_id_mails_mail_id_delete(corner_id, mail_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**corner_id** | **String** |  | [required] |
**mail_id** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## corners_corner_id_mails_post

> corners_corner_id_mails_post(corner_id, corners_corner_id_mails_post_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**corner_id** | **String** |  | [required] |
**corners_corner_id_mails_post_request** | [**CornersCornerIdMailsPostRequest**](CornersCornerIdMailsPostRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

