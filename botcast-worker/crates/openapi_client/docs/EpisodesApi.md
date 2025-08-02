# \EpisodesApi

All URIs are relative to *http://localhost:1234*

Method | HTTP request | Description
------------- | ------------- | -------------
[**episodes_episode_id_delete**](EpisodesApi.md#episodes_episode_id_delete) | **DELETE** /episodes/{episodeId} | 
[**episodes_episode_id_get**](EpisodesApi.md#episodes_episode_id_get) | **GET** /episodes/{episodeId} | 
[**episodes_episode_id_put**](EpisodesApi.md#episodes_episode_id_put) | **PUT** /episodes/{episodeId} | 
[**episodes_post**](EpisodesApi.md#episodes_post) | **POST** /episodes | 



## episodes_episode_id_delete

> episodes_episode_id_delete(episode_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**episode_id** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## episodes_episode_id_get

> models::Episode episodes_episode_id_get(episode_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**episode_id** | **String** |  | [required] |

### Return type

[**models::Episode**](Episode.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## episodes_episode_id_put

> episodes_episode_id_put(episode_id, episodes_episode_id_put_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**episode_id** | **String** |  | [required] |
**episodes_episode_id_put_request** | [**EpisodesEpisodeIdPutRequest**](EpisodesEpisodeIdPutRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## episodes_post

> episodes_post(episodes_post_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**episodes_post_request** | [**EpisodesPostRequest**](EpisodesPostRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

