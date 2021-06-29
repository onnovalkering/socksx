use crate::addresses::ProxyAddress;
use crate::socks6::options::{MetadataOption, SocksOption};

#[derive(Clone, Debug)]
pub struct SocksChain {
    pub index: usize,
    pub links: Vec<ProxyAddress>,
}

impl Default for SocksChain {
    fn default() -> Self {
        Self::new(0, vec![])
    }
}

impl SocksChain {
    pub fn new(
        index: usize,
        links: Vec<ProxyAddress>,
    ) -> Self {
        Self { index, links }
    }

    ///
    ///
    ///
    pub fn current_link(&self) -> &ProxyAddress {
        self.links.get(self.index).unwrap()
    }

    ///
    ///
    ///
    pub fn has_next(&self) -> bool {
        self.index + 1 < self.links.len()
    }    

    ///
    ///
    ///
    pub fn next_link(&mut self) -> Option<&ProxyAddress> {
        let link = self.links.get(self.index + 1);
        if link.is_some() {
            self.index += 1;
        }

        link
    }

    ///
    ///
    ///
    pub fn detour(
        &mut self,
        links: &[ProxyAddress],
    ) {
        let links = links.iter().cloned();

        if self.links.is_empty() {
            // This means we're currently at the root.
            // We'll append ourself as the root link.
            self.links.push(ProxyAddress::root());
            self.links.extend(links);
        } else {
            let position = self.index + 1..self.index + 1;
            self.links.splice(position, links);
        }
    }

    ///
    ///
    ///
    pub fn as_options(&self) -> Vec<SocksOption> {
        let mut chain_options: Vec<SocksOption> = self
            .links
            .iter()
            .enumerate()
            .map(|(i, c)| (i as u16, c.to_string()))
            .map(|(i, c)| MetadataOption::new(1000 + i, c).wrap())
            .collect();

        chain_options.push(MetadataOption::new(998, self.index.to_string()).wrap());
        chain_options.push(MetadataOption::new(999, self.links.len().to_string()).wrap());

        chain_options
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn abc() {
        let mut chain = SocksChain::new(
            1,
            vec![
                ProxyAddress::new(6, String::from("localhost"), 1, None),
                ProxyAddress::new(6, String::from("localhost"), 2, None),
                ProxyAddress::new(6, String::from("localhost"), 3, None),
            ],
        );

        let extra = vec![
            ProxyAddress::new(6, String::from("localhost"), 4, None),
            ProxyAddress::new(6, String::from("localhost"), 5, None),
        ];
        chain.detour(&extra);

        let order: Vec<u16> = chain.links.iter().map(|l| l.port).collect();
        assert_eq!(order, vec![1, 2, 4, 5, 3]);
    }
}
