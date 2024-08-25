use ieee80211::{
    common::Empty,
    element_chain,
    elements::{
        element_chain::{ChainElement, ElementChainEnd, ElementChainLink},
        rates::{EncodedRate, SupportedRatesElement},
        DSSSParameterSetElement, SSIDElement,
    },
    ssid, supported_rates,
};

// These constants are used to test the return type of the element_chain! macro and ensure const compatibility.

const SSID_ELEMENT: SSIDElement = ssid!("Test");
const EMPTY_ELEMENT_CHAIN: Empty = element_chain! {};
const SINGLE_ELEMENT_CHAIN: ElementChainEnd<SSIDElement> = element_chain! {
    SSID_ELEMENT
};
const MULTI_ELEMENT_CHAIN: ElementChainLink<
    SSIDElement,
    ElementChainLink<
        SupportedRatesElement<[EncodedRate; 2]>,
        ElementChainEnd<DSSSParameterSetElement>,
    >,
> = element_chain! {
    SSID_ELEMENT,
    supported_rates![
        1,
        1.5 B
    ],
    DSSSParameterSetElement {
        current_channel: 1
    }
};

#[test]
fn test_element_chain() {
    let chain: ElementChainEnd<SSIDElement> = ElementChainEnd::new(SSID_ELEMENT);
    let chain: ElementChainLink<
        SSIDElement,
        ElementChainEnd<SupportedRatesElement<[EncodedRate; 2]>>,
    > = chain.append(supported_rates![1, 1.5 B]);
}
