# Frequently asked questions

- [Frequently asked questions](#frequently-asked-questions)
  - [How is the database maintained?](#how-is-the-database-maintained)
  - [How does conflict resolution work?](#how-does-conflict-resolution-work)
  - [How are publishers found?](#how-are-publishers-found)
  - [How does a user know which publisher to trust?](#how-does-a-user-know-which-publisher-to-trust)
  - [Can publishers be censored?](#can-publishers-be-censored)
  - [What if the IPNS key is lost?](#what-if-the-ipns-key-is-lost)


## How is the database maintained?

The library can be run as a background process with three parts:

- Update local data
- Publish to IPFS
- Update IPNS to latest IPFS hash.

Updating data has two components:

- Check for other publishers. Pull in data if available.
- Check for local raw data. If there is a local raw data source
being created (e.g., new text files being added as users submit them to a frontend etc.)
then they may be incorporated into the database once there is enough data to publish
a Volume.

## How does conflict resolution work?

Conflict resolution can be handled programmatically and there are different options:

- Indiscriminantly accept and build upon other publishers.
- Employ specific filtering to extract data from other publishers.
- Search and re-org data according to some criteria.
- Ignore other publishers completely.

Anyone can survey already published manifests and choose to build atop those.
Or if they consider the data to be bad, they can publish a competing manifest.

For example someone is maintaining an address nametags database. They find a second
publisher doing the same, but want to filter out some of the nametags they disagree with.

They can either extract data and publish a completely different manifest, or only change
data after a certain volume (e.g., change some entries in recent data.). The latter is akin
to a re-org, with the manifests largely containing identical CIDs.

## How are publishers found?

A publisher creates an IPNS using a private key. They then submit that IPNS to a smart contract
to associate it with a particular topic.

A user can check the contract and search a topic to get a list of IPNS. They then can find a
manifest at each IPNS and compare them.

## How does a user know which publisher to trust?

A user may find a publisher out of band (twitter, discord, etc.). They may instead choose
to check the broadcasting contract for a particular topic. After getting a list of publisher IPNS
for that topic, they can get the manifest from each.

Selection can be made using different criteria, and the process may be automated. Strategies include:
- Pick the manifest with the most number of published Volumes
- Check the manifest for additional notes/comments by the publisher
- Try different manifests, sampling for particular data to see if it is present.

## Can publishers be censored?

No one can censor a publisher, they can only be ignored. The IPNSs are posted once to a smart contract for discovery.

## What if the IPNS key is lost?

The publisher can generate a new IPNS key and submit that to the broadcasting contract as a transaction.