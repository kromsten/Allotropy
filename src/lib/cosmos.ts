import type { TokenizeShareRecord, ChainQueryClient } from "$types";
import { 
    QueryTokenizeShareRecordsOwnedRequest, 
    QueryTokenizeShareRecordsOwnedResponse
} from "persistenceonejs/gaia/liquid/v1beta1/query";






// patj  /gaia.liquid.v1beta1.Query/QueryTokenizeShareRecordsOwnedRequest


export const queryLiquidRecords = async (
    queryClient: ChainQueryClient,
    owner: string
): Promise<any> => {

    const response = await queryClient.queryAbci(
        '/gaia.liquid.v1beta1.Query/TokenizeShareRecordsOwned',
        QueryTokenizeShareRecordsOwnedRequest.encode({ owner }).finish()
    )
    
    const decodedResponse = QueryTokenizeShareRecordsOwnedResponse.decode(response.value);
    console.log('Decoded Response:', decodedResponse);
    return decodedResponse;
}