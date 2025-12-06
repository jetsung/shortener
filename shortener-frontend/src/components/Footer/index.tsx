import React from 'react';
import { Typography } from '@douyinfe/semi-ui-19';
import { IconGithubLogo, IconGitlabLogo, IconHome } from '@douyinfe/semi-icons';

const { Text } = Typography;

const Footer: React.FC = () => {
  return (
    <div
      style={{
        textAlign: 'center',
        padding: '8px 0',
        borderTop: '1px solid var(--semi-color-border)',
        backgroundColor: 'var(--semi-color-bg-1)',
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        flexWrap: 'wrap',
        gap: '16px',
      }}
    >
      <a
        href="https://forum.idev.top"
        target="_blank"
        rel="noopener noreferrer"
        style={{
          color: 'var(--semi-color-primary)',
          textDecoration: 'none',
          display: 'flex',
          alignItems: 'center',
        }}
      >
        <IconHome style={{ marginRight: 4 }} />
        iDEV Forum
      </a>
      <a
        href="https://git.jetsung.com/jetsung/shortener"
        target="_blank"
        rel="noopener noreferrer"
        style={{
          color: 'var(--semi-color-primary)',
          textDecoration: 'none',
          display: 'flex',
          alignItems: 'center',
        }}
      >
        <IconGitlabLogo style={{ marginRight: 4 }} />
        Shortener Code
      </a>
      <a
        href="https://github.com/idev-sig/shortener-server"
        target="_blank"
        rel="noopener noreferrer"
        style={{
          color: 'var(--semi-color-primary)',
          textDecoration: 'none',
          display: 'flex',
          alignItems: 'center',
        }}
      >
        <IconGithubLogo style={{ marginRight: 4 }} />
        Shortener GitHub
      </a>
      <Text type="tertiary" size="small">
        © 2025 Shortener
      </Text>
    </div>
  );
};

export default Footer;
